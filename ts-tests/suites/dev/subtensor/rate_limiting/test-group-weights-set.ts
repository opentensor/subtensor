import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_WEIGHTS_SET,
    disableAdminFreezeWindow,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_WEIGHTS_SET_01",
    title: "Rate-limiting: GROUP_WEIGHTS_SET",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let alice: KeyringPair;
        let bob: KeyringPair;
        let netuid: number;

        beforeAll(() => {
            polkadotJs = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
        });

        it({
            id: "T01",
            title: "set_weights and commit_weights share the per-netuid weights window",
            test: async () => {
                const SPAN = 5;
                await disableAdminFreezeWindow(polkadotJs, context, alice);

                // Subnet setup: alice owns the subnet, bob is the validator hotkey.
                const registerNet = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await registerNet.signAsync(alice)]);
                const added = ((await polkadotJs.query.system.events()) as any).find(
                    (e: any) => e.event.method === "NetworkAdded"
                );
                expect(added).to.not.be.undefined;
                netuid = Number((added as any).event.data[0].toString());

                // Make weight-setting permissive on this dev subnet.
                const tweaks = [
                    polkadotJs.tx.adminUtils.sudoSetStakeThreshold(0),
                    polkadotJs.tx.adminUtils.sudoSetMinAllowedWeights(netuid, 0),
                    polkadotJs.tx.adminUtils.sudoSetTempo(netuid, 1),
                    polkadotJs.tx.adminUtils.sudoSetCommitRevealWeightsEnabled(netuid, false),
                ];
                for (const t of tweaks) {
                    await context.createBlock([await polkadotJs.tx.sudo.sudo(t).signAsync(alice)]);
                }

                // Register bob as a neuron on this subnet so set_weights is dispatchable.
                const burnedReg = polkadotJs.tx.subtensorModule.burnedRegister(netuid, bob.address);
                await context.createBlock([await burnedReg.signAsync(alice)]);

                // Wire weight calls into the group and reshape span for fast tests.
                const sampleSet = polkadotJs.tx.subtensorModule.setWeights(netuid, [0], [1], 0);
                const sampleCommit = polkadotJs.tx.subtensorModule.commitWeights(
                    netuid,
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                );
                await ensureCallInGroup(polkadotJs, context, alice, sampleSet, GROUP_WEIGHTS_SET);
                await ensureCallInGroup(polkadotJs, context, alice, sampleCommit, GROUP_WEIGHTS_SET);
                await setGroupSpan(polkadotJs, context, alice, GROUP_WEIGHTS_SET, netuid, SPAN);

                // 1. Initial set_weights → succeeds, records usage by neuron.
                const first = polkadotJs.tx.subtensorModule.setWeights(netuid, [0], [1], 0);
                await expectExtrinsicOk(polkadotJs, context, await first.signAsync(bob), "set_weights_initial");

                // 2. Immediate set_weights again → blocked.
                const early = polkadotJs.tx.subtensorModule.setWeights(netuid, [0], [1], 0);
                await expectRateLimited(polkadotJs, context, await early.signAsync(bob), "set_weights_in_window");

                // 3. Age out → allowed.
                await seal(context, SPAN);
                const after = polkadotJs.tx.subtensorModule.setWeights(netuid, [0], [1], 0);
                await expectExtrinsicOk(polkadotJs, context, await after.signAsync(bob), "set_weights_after_window");
            },
        });
    },
});
