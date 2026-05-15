import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    addNewSubnetwork,
    addStake,
    forceSetBalance,
    generateKeyringPair,
    getBalance,
    getStake,
    removeStake,
    startCall,
    sudoSetLockReductionInterval,
    tao,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "04_remove_stake",
    title: "▶ remove_stake extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;
        let netuid: number;
        const hotkey = generateKeyringPair("sr25519");
        const coldkey = generateKeyringPair("sr25519");
        const hotkeyAddress = hotkey.address;
        const coldkeyAddress = coldkey.address;

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
            title: "should remove stake from a hotkey",
            test: async () => {
                // Add stake first
                await addStake(api, coldkey, hotkeyAddress, netuid, tao(200));

                // Get initial stake and balance (converted from U64F64 for display)
                const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                const balanceBefore = await getBalance(api, coldkeyAddress);
                expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

                // Remove stake (amount is in alpha units - use raw U64F64 value)
                const stake = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
                const unstakeAmount = stake / 2n;
                await removeStake(api, coldkey, hotkeyAddress, netuid, unstakeAmount);

                // Verify balance increased (received TAO from unstaking)
                const balanceAfter = await getBalance(api, coldkeyAddress);
                expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

                log("✅ Successfully removed stake.");
            },
        });
    },
});
