import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_DELEGATE_TAKE,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_DELEGATE_TAKE_01",
    title: "Rate-limiting: GROUP_DELEGATE_TAKE",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let alice: KeyringPair;
        let bob: KeyringPair;

        beforeAll(() => {
            polkadotJs = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
        });

        it({
            id: "T01",
            title: "increase_take rejected inside window; allowed after; decrease_take bypasses enforcement",
            test: async () => {
                const SPAN = 5;

                // Establish bob as alice's hotkey via register_network + addStake.
                const registerTx = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await registerTx.signAsync(alice)]);
                const added = ((await polkadotJs.query.system.events()) as any).find(
                    (e: any) => e.event.method === "NetworkAdded"
                );
                expect(added).to.not.be.undefined;

                // `increase_take` itself creates the Delegates entry on first call — no separate
                // `become_delegate` extrinsic exists anymore.

                // Wire calls into the group and reduce span.
                const sampleInc = polkadotJs.tx.subtensorModule.increaseTake(bob.address, 100);
                const sampleDec = polkadotJs.tx.subtensorModule.decreaseTake(bob.address, 1);
                await ensureCallInGroup(polkadotJs, context, alice, sampleInc, GROUP_DELEGATE_TAKE);
                await ensureCallInGroup(polkadotJs, context, alice, sampleDec, GROUP_DELEGATE_TAKE);
                await setGroupSpan(polkadotJs, context, alice, GROUP_DELEGATE_TAKE, null, SPAN);

                const current = (await polkadotJs.query.subtensorModule.delegates(bob.address)) as any;
                const baseTake = Number(current?.toString?.() ?? 0);
                if (baseTake === 0) {
                    throw new Error("expected Delegates[bob] to have a positive default take");
                }

                const dec = polkadotJs.tx.subtensorModule.decreaseTake(bob.address, baseTake - 1);
                await expectExtrinsicOk(polkadotJs, context, await dec.signAsync(alice), "decrease_take_arms_window");

                // 1. Immediate increase back to baseTake → blocked (LastSeen just written).
                const earlyInc = polkadotJs.tx.subtensorModule.increaseTake(bob.address, baseTake);
                await expectRateLimited(
                    polkadotJs,
                    context,
                    await earlyInc.signAsync(alice),
                    "increase_take_in_window"
                );

                // 2. Another decrease — `bypass_and_record` for `new <= current` → success despite window.
                const decAgain = polkadotJs.tx.subtensorModule.decreaseTake(bob.address, baseTake - 2);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await decAgain.signAsync(alice),
                    "decrease_take_bypasses_window"
                );

                // 3. Age out and increase → allowed.
                await seal(context, SPAN);
                const afterInc = polkadotJs.tx.subtensorModule.increaseTake(bob.address, baseTake - 1);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await afterInc.signAsync(alice),
                    "increase_take_after_window"
                );
            },
        });
    },
});
