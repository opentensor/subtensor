import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_SWAP_KEYS,
    disableAdminFreezeWindow,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_SWAP_KEYS_01",
    title: "Rate-limiting: GROUP_SWAP_KEYS",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let alice: KeyringPair;
        let bob: KeyringPair;
        let charlie: KeyringPair;
        let dave: KeyringPair;

        beforeAll(() => {
            polkadotJs = context.polkadotJs();
            alice = context.keyring.alice;
            bob = context.keyring.bob;
            charlie = context.keyring.charlie;
            dave = (context.keyring as any).dave ?? context.keyring.charlie;
        });

        it({
            id: "T01",
            title: "swap_hotkey within window is rejected; allowed after the window expires",
            test: async () => {
                const SPAN = 5;
                await disableAdminFreezeWindow(polkadotJs, context, alice);

                // Establish OwnedHotkeys[(alice, bob)] by registering a subnet with bob as the
                // owner hotkey, then opt bob in via add_stake.
                const registerTx = polkadotJs.tx.subtensorModule.registerNetwork(bob.address);
                await context.createBlock([await registerTx.signAsync(alice)]);
                const evs = (await polkadotJs.query.system.events()) as any;
                const added = evs.find((e: any) => e.event.method === "NetworkAdded");
                expect(added).to.not.be.undefined;
                const netuid = Number((added as any).event.data[0].toString());

                const enableTx = polkadotJs.tx.adminUtils.sudoSetSubtokenEnabled(netuid, true);
                await context.createBlock([await polkadotJs.tx.sudo.sudo(enableTx).signAsync(alice)]);

                const stakeTx = polkadotJs.tx.subtensorModule.addStake(bob.address, netuid, 1_000_000_000_000);
                await context.createBlock([await stakeTx.signAsync(alice)]);

                // Wire swap calls into the group and reduce span for fast tests.
                const sample = polkadotJs.tx.subtensorModule.swapHotkey(bob.address, charlie.address, null);
                await ensureCallInGroup(polkadotJs, context, alice, sample, GROUP_SWAP_KEYS);
                await setGroupSpan(polkadotJs, context, alice, GROUP_SWAP_KEYS, null, SPAN);

                // 1. Successful swap → records LastSeen for usage key Account(alice).
                const first = polkadotJs.tx.subtensorModule.swapHotkey(bob.address, charlie.address, null);
                await expectExtrinsicOk(polkadotJs, context, await first.signAsync(alice), "swap_hotkey_initial");

                // 2. Same coldkey, immediate swap → blocked.
                const early = polkadotJs.tx.subtensorModule.swapHotkey(charlie.address, dave.address, null);
                await expectRateLimited(polkadotJs, context, await early.signAsync(alice), "swap_hotkey_in_window");

                // 3. Age out: seal `span - 1` blocks → allowed.
                await seal(context, SPAN - 1);
                const after = polkadotJs.tx.subtensorModule.swapHotkey(charlie.address, dave.address, null);
                await expectExtrinsicOk(polkadotJs, context, await after.signAsync(alice), "swap_hotkey_after_window");
            },
        });
    },
});
