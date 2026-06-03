import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_SERVE,
    disableAdminFreezeWindow,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_SERVE_01",
    title: "Rate-limiting: GROUP_SERVE",
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
            title: "serve_axon and serve_axon_tls share the Axon usage key; serve_prometheus separate",
            test: async () => {
                const SPAN = 5;
                // GROUP_SERVE uses `RootOrSubnetOwnerAdminWindow` rule for limit-setting — needs an
                // open admin window even under sudo.
                await disableAdminFreezeWindow(polkadotJs, context, alice);

                // Subnet with bob registered as a neuron (required by `serve_axon` etc.).
                const registerTx = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await registerTx.signAsync(alice)]);
                const added = ((await polkadotJs.query.system.events()) as any).find(
                    (e: any) => e.event.method === "NetworkAdded"
                );
                expect(added).to.not.be.undefined;
                netuid = Number((added as any).event.data[0].toString());

                // Register bob as a neuron on this subnet.
                const burnedReg = polkadotJs.tx.subtensorModule.burnedRegister(netuid, bob.address);
                await context.createBlock([await burnedReg.signAsync(alice)]);

                // Wire serve calls into the group and reshape span.
                const sampleAxon = polkadotJs.tx.subtensorModule.serveAxon(netuid, 1, 0, 3030, 4, 0, 0, 0);
                const sampleAxonTls = polkadotJs.tx.subtensorModule.serveAxonTls(
                    netuid,
                    1,
                    0,
                    3030,
                    4,
                    0,
                    0,
                    0,
                    "0x" + "07".repeat(130)
                );
                const samplePrometheus = polkadotJs.tx.subtensorModule.servePrometheus(netuid, 1, 1, 3031, 4);
                await ensureCallInGroup(polkadotJs, context, alice, sampleAxon, GROUP_SERVE);
                await ensureCallInGroup(polkadotJs, context, alice, sampleAxonTls, GROUP_SERVE);
                await ensureCallInGroup(polkadotJs, context, alice, samplePrometheus, GROUP_SERVE);
                await setGroupSpan(polkadotJs, context, alice, GROUP_SERVE, netuid, SPAN);

                // 1. serve_axon succeeds → records usage at AccountSubnetServing{Axon}.
                const axon1 = polkadotJs.tx.subtensorModule.serveAxon(netuid, 1, 0, 3030, 4, 0, 0, 0);
                await expectExtrinsicOk(polkadotJs, context, await axon1.signAsync(bob), "serve_axon_initial");

                // 2. serve_axon_tls shares the Axon usage key → blocked inside window.
                const tlsEarly = polkadotJs.tx.subtensorModule.serveAxonTls(
                    netuid,
                    1,
                    0,
                    3030,
                    4,
                    0,
                    0,
                    0,
                    "0x" + "07".repeat(130)
                );
                await expectRateLimited(polkadotJs, context, await tlsEarly.signAsync(bob), "serve_axon_tls_in_window");

                // 3. serve_prometheus uses a SEPARATE usage key (endpoint=Prometheus) → allowed
                //    even though the group's config is shared.
                const prom = polkadotJs.tx.subtensorModule.servePrometheus(netuid, 1, 1, 3031, 4);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await prom.signAsync(bob),
                    "serve_prometheus_separate_key"
                );

                // 4. After window expires → axon_tls allowed.
                await seal(context, SPAN);
                const tlsAfter = polkadotJs.tx.subtensorModule.serveAxonTls(
                    netuid,
                    1,
                    0,
                    3030,
                    4,
                    0,
                    0,
                    0,
                    "0x" + "07".repeat(130)
                );
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await tlsAfter.signAsync(bob),
                    "serve_axon_tls_after_window"
                );
            },
        });
    },
});
