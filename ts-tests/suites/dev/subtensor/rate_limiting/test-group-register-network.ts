import { beforeAll, describeSuite } from "@moonwall/cli";
import type { KeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
    GROUP_REGISTER_NETWORK,
    ensureCallInGroup,
    expectExtrinsicOk,
    expectRateLimited,
    setGroupSpan,
} from "./_utils.ts";
import { seal } from "../../../../utils/dev_utils.ts";

describeSuite({
    id: "DEV_SUB_RL_REGISTER_NETWORK_01",
    title: "Rate-limiting: GROUP_REGISTER_NETWORK",
    foundationMethods: "dev",
    testCases: ({ it, context }) => {
        let polkadotJs: ApiPromise;
        let alice: KeyringPair;

        beforeAll(() => {
            polkadotJs = context.polkadotJs();
            alice = context.keyring.alice;
        });

        it({
            id: "T01",
            title: "register_network and register_network_with_identity share a global window",
            test: async () => {
                const SPAN = 5;
                const sample = polkadotJs.tx.subtensorModule.registerNetwork(alice.address);
                const sampleWithIdentity = polkadotJs.tx.subtensorModule.registerNetworkWithIdentity(
                    alice.address,
                    null
                );

                // Bring chain into post-migration state for these two calls.
                await ensureCallInGroup(polkadotJs, context, alice, sample, GROUP_REGISTER_NETWORK);
                await ensureCallInGroup(polkadotJs, context, alice, sampleWithIdentity, GROUP_REGISTER_NETWORK);
                await setGroupSpan(polkadotJs, context, alice, GROUP_REGISTER_NETWORK, null, SPAN);

                // 1. First registration succeeds → LastSeen written.
                const first = polkadotJs.tx.subtensorModule.registerNetwork(alice.address);
                await expectExtrinsicOk(polkadotJs, context, await first.signAsync(alice), "first_register");

                // 2. Sibling call in the same group is rejected immediately (delta < span).
                const sibling = polkadotJs.tx.subtensorModule.registerNetworkWithIdentity(alice.address, null);
                await expectRateLimited(polkadotJs, context, await sibling.signAsync(alice), "sibling_in_window");

                // 3. Age out window: seal `span - 1` empty blocks, still inside → rejected.
                await seal(context, SPAN - 2);
                const stillBlocked = polkadotJs.tx.subtensorModule.registerNetwork(alice.address);
                await expectRateLimited(polkadotJs, context, await stillBlocked.signAsync(alice), "still_in_window");

                // 4. One more empty block → delta == span → allowed.
                await seal(context, 1);
                const afterWindow = polkadotJs.tx.subtensorModule.registerNetwork(alice.address);
                await expectExtrinsicOk(polkadotJs, context, await afterWindow.signAsync(alice), "after_window");
            },
        });
    },
});
