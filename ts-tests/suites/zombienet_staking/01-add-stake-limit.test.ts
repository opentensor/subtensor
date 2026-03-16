import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { generateKeyringPair } from "@moonwall/util";
import { addNewSubnetwork, addStakeLimit, forceSetBalance, getStake, startCall, tao } from "../../utils";

describeSuite({
    id: "01_add_stake_limit",
    title: "▶ add_stake_limit extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        const hotkey = generateKeyringPair("sr25519");
        const coldkey = generateKeyringPair("sr25519");
        const hotkeyAddress = hotkey.address;
        const coldkeyAddress = coldkey.address;
        let netuid: number;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
            await forceSetBalance(api, hotkeyAddress);
            await forceSetBalance(api, coldkeyAddress);
            netuid = await addNewSubnetwork(api, hotkey, coldkey);
            await startCall(api, netuid, coldkey);
        });

        it({
            id: "T01",
            title: "should add stake with price limit (allow partial)",
            test: async () => {
                // Get initial stake
                const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

                // Add stake with limit price and allow partial fills, limit_price is MAX TAO per Alpha willing to pay.
                const stakeAmount = tao(44);
                const limitPrice = tao(6);
                await addStakeLimit(api, coldkey, hotkeyAddress, netuid, stakeAmount, limitPrice, true);

                // Verify stake increased
                const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                expect(stakeAfter, "Stake should increase").toBeGreaterThan(stakeBefore);

                log("✅ Successfully added stake with limit (allow partial).");
            },
        });

        it({
            id: "T02",
            title: "should add stake with price limit (fill or kill)",
            test: async () => {
                // Get initial stake
                const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

                // Add stake with limit price (fill or kill mode), limit_price is MAX TAO per Alpha willing to pay
                const stakeAmount = tao(44);
                const limitPrice = tao(6);
                await addStakeLimit(api, coldkey, hotkeyAddress, netuid, stakeAmount, limitPrice, false);

                // Verify stake increased
                const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                expect(stakeAfter, "Stake should increase").toBeGreaterThan(stakeBefore);

                log("✅ Successfully added stake with limit (fill or kill).");
            },
        });
    },
});
