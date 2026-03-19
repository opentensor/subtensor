import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    addNewSubnetwork,
    addStakeLimit,
    forceSetBalance,
    generateKeyringPair,
    getStake,
    startCall,
    sudoSetLockReductionInterval,
    tao,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "01_add_stake_limit",
    title: "▶ add_stake_limit extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;

        const hotkey = generateKeyringPair("sr25519");
        const coldkey = generateKeyringPair("sr25519");
        const hotkeyAddress = hotkey.address;
        const coldkeyAddress = coldkey.address;
        let netuid: number;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
            await forceSetBalance(api, hotkeyAddress);
            await forceSetBalance(api, coldkeyAddress);
            await sudoSetLockReductionInterval(api, 1);
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
