import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_OWNER_HPARAMS,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_OWNER_HPARAMS_01",
    title: "Rate-limiting: GROUP_OWNER_HPARAMS",
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
            title: "ConfigOnly: shared limit, per-transaction usage; tempo-scaled span via adjust_span",
            test: async () => {
                // Setup: alice as subnet owner; tempo=1 so adjust_span keeps the effective span small.
                const registerTx = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await registerTx.signAsync(alice)]);
                const added = ((await polkadotJs.query.system.events()) as any).find(
                    (e: any) => e.event.method === "NetworkAdded"
                );
                expect(added).to.not.be.undefined;
                netuid = Number((added as any).event.data[0].toString());

                const setTempo = polkadotJs.tx.adminUtils.sudoSetTempo(netuid, 1);
                await context.createBlock([await polkadotJs.tx.sudo.sudo(setTempo).signAsync(alice)]);

                // adjust_span multiplies the configured span by Tempo(netuid). With tempo=1 the
                // effective span equals the stored value.
                const SPAN_RAW = 5;
                const sampleKappa = polkadotJs.tx.adminUtils.sudoSetKappa(netuid, 32_768);
                const sampleRho = polkadotJs.tx.adminUtils.sudoSetRho(netuid, 30);
                await ensureCallInGroup(polkadotJs, context, alice, sampleKappa, GROUP_OWNER_HPARAMS);
                await ensureCallInGroup(polkadotJs, context, alice, sampleRho, GROUP_OWNER_HPARAMS);
                await setGroupSpan(polkadotJs, context, alice, GROUP_OWNER_HPARAMS, null, SPAN_RAW);

                // 1. sudo wrap: root bypasses, so we go through alice as subnet owner (regular
                //    signed origin) which IS rate-limited via the admin-window rule.
                const kappa1 = polkadotJs.tx.adminUtils.sudoSetKappa(netuid, 32_768);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await polkadotJs.tx.sudo.sudo(kappa1).signAsync(alice),
                    "set_kappa_initial"
                );

                // 2. Repeating kappa in the same window → blocked (usage is per transaction).
                const kappa2 = polkadotJs.tx.adminUtils.sudoSetKappa(netuid, 30_000);
                await expectRateLimited(
                    polkadotJs,
                    context,
                    await polkadotJs.tx.sudo.sudo(kappa2).signAsync(alice),
                    "set_kappa_in_window"
                );

                // 3. ConfigOnly: a DIFFERENT hparam call has its own LastSeen → allowed even though
                //    the group's config is shared.
                const rho = polkadotJs.tx.adminUtils.sudoSetRho(netuid, 31);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await polkadotJs.tx.sudo.sudo(rho).signAsync(alice),
                    "set_rho_separate_transaction"
                );

                // 4. After window expires → kappa allowed again.
                await seal(context, SPAN_RAW);
                const kappa3 = polkadotJs.tx.adminUtils.sudoSetKappa(netuid, 28_000);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await polkadotJs.tx.sudo.sudo(kappa3).signAsync(alice),
                    "set_kappa_after_window"
                );
            },
        });
    },
});
