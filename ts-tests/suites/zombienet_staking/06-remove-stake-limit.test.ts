import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    addNewSubnetwork,
    addStake,
    forceSetBalance,
    generateKeyringPair,
    getBalance,
    getStake,
    removeStakeLimit,
    startCall,
    sudoSetLockReductionInterval,
    tao,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "06_remove_stake_limit",
    title: "▶ remove_stake_limit extrinsic",
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
            await sudoSetLockReductionInterval(api, 1);
            await forceSetBalance(api, hotkeyAddress);
            await forceSetBalance(api, coldkeyAddress);
            netuid = await addNewSubnetwork(api, hotkey, coldkey);
            await startCall(api, netuid, coldkey);
        });

        it({
            id: "T01",
            title: "should remove stake with price limit (allow partial)",
            test: async () => {
                // Add stake first (100 TAO like benchmark)
                await addStake(api, coldkey, hotkeyAddress, netuid, tao(100));

                // Get initial stake and balance
                const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                const balanceBefore = await getBalance(api, coldkeyAddress);
                log(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
                expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

                // Remove stake with limit price and allow partial fills
                const unstakeAmount = tao(30);
                const limitPrice = tao(1);
                await removeStakeLimit(api, coldkey, hotkeyAddress, netuid, unstakeAmount, limitPrice, true);

                // Verify balance increased (received TAO from unstaking)
                const balanceAfter = await getBalance(api, coldkeyAddress);
                expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

                log("✅ Successfully removed stake with limit (allow partial).");
            },
        });

        it({
            id: "T02",
            title: "should remove stake with price limit (fill or kill)",
            test: async () => {
                // Add stake first (100 TAO like benchmark)
                await addStake(api, coldkey, hotkeyAddress, netuid, tao(100));

                // Get initial stake and balance
                const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                const balanceBefore = await getBalance(api, coldkeyAddress);
                log(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
                expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

                // Remove stake with limit price (fill or kill mode)
                const unstakeAmount = tao(30);
                const limitPrice = tao(1);
                await removeStakeLimit(api, coldkey, hotkeyAddress, netuid, unstakeAmount, limitPrice, false);

                // Verify balance increased (received TAO from unstaking)
                const balanceAfter = await getBalance(api, coldkeyAddress);
                expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

                log("✅ Successfully removed stake with limit (fill or kill).");
            },
        });
    },
});
