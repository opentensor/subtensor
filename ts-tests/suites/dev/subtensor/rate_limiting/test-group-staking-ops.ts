import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_STAKING_OPS,
    disableAdminFreezeWindow,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_STAKING_OPS_01",
    title: "Rate-limiting: GROUP_STAKING_OPS",
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
            title: "add_stake records usage; remove_stake blocked while inside window",
            test: async () => {
                const SPAN = 5;

                // Subnet hparam toggles require an open admin window.
                await disableAdminFreezeWindow(polkadotJs, context, alice);

                // 1. Create a fresh subnet owned by alice, with bob as hotkey.
                const registerTx = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await registerTx.signAsync(alice)]);
                const evs = (await polkadotJs.query.system.events()) as any;
                const added = evs.find((e: any) => e.event.method === "NetworkAdded");
                expect(added).to.not.be.undefined;
                netuid = Number((added as any).event.data[0].toString());

                // 2. Enable subtoken for staking.
                const enableTx = polkadotJs.tx.adminUtils.sudoSetSubtokenEnabled(netuid, true);
                await context.createBlock([await polkadotJs.tx.sudo.sudo(enableTx).signAsync(alice)]);

                // 3. Wire staking calls into the group, then override span for fast tests.
                const sampleAdd = polkadotJs.tx.subtensorModule.addStake(bob.address, netuid, 1_000_000_000);
                const sampleRemove = polkadotJs.tx.subtensorModule.removeStake(bob.address, netuid, 100_000_000);
                await ensureCallInGroup(polkadotJs, context, alice, sampleAdd, GROUP_STAKING_OPS);
                // remove_stake is read-only in the production migration (`migrations/rate_limiting.rs`).
                await ensureCallInGroup(polkadotJs, context, alice, sampleRemove, GROUP_STAKING_OPS, true);
                await setGroupSpan(polkadotJs, context, alice, GROUP_STAKING_OPS, null, SPAN);

                // 4. Successful add_stake → writes LastSeen (add_stake is NOT read-only).
                const addTx = polkadotJs.tx.subtensorModule.addStake(bob.address, netuid, 1_000_000_000_000);
                await expectExtrinsicOk(polkadotJs, context, await addTx.signAsync(alice), "add_stake_initial");

                // 5. Immediate remove_stake on the same (coldkey, hotkey, netuid) → rate-limited.
                const earlyRemove = polkadotJs.tx.subtensorModule.removeStake(bob.address, netuid, 100_000_000);
                await expectRateLimited(
                    polkadotJs,
                    context,
                    await earlyRemove.signAsync(alice),
                    "remove_stake_in_window"
                );

                // 6. Seal `span - 2` more empty blocks → still inside the window.
                await seal(context, SPAN - 2);
                const stillBlocked = polkadotJs.tx.subtensorModule.removeStake(bob.address, netuid, 100_000_000);
                await expectRateLimited(
                    polkadotJs,
                    context,
                    await stillBlocked.signAsync(alice),
                    "remove_stake_just_before_window"
                );

                // 7. One more empty block → delta == span → allowed.
                await seal(context, 1);
                const afterWindow = polkadotJs.tx.subtensorModule.removeStake(bob.address, netuid, 100_000_000);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await afterWindow.signAsync(alice),
                    "remove_stake_after_window"
                );
            },
        });

        it({
            id: "T02",
            title: "remove_stake is read-only: success does NOT reset the group window",
            test: async () => {
                const SPAN = 5;
                await setGroupSpan(polkadotJs, context, alice, GROUP_STAKING_OPS, null, SPAN);

                // The previous test left the chain past the window with a successful remove_stake.
                // remove_stake is marked `read_only` → it does NOT write LastSeen, so a subsequent
                // add_stake/remove_stake should NOT be locked out by it. Seal one block then try.
                await seal(context, 1);
                const removeAgain = polkadotJs.tx.subtensorModule.removeStake(bob.address, netuid, 100_000_000);
                await expectExtrinsicOk(
                    polkadotJs,
                    context,
                    await removeAgain.signAsync(alice),
                    "remove_stake_read_only_does_not_extend"
                );
            },
        });
    },
});
