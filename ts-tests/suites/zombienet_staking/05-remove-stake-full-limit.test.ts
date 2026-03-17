import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
    addNewSubnetwork,
    addStake,
    burnedRegister,
    forceSetBalance,
    generateKeyringPair,
    getBalance,
    getStake,
    removeStakeFullLimit,
    startCall,
    sudoSetAdminFreezeWindow,
    sudoSetTempo,
    tao,
} from "../../utils";

describeSuite({
    id: "05_remove_stake_full_limit",
    title: "▶ remove_stake_full_limit extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        const ownerHotkey = generateKeyringPair("sr25519");
        const stakerHotkey = generateKeyringPair("sr25519");
        const coldkey = generateKeyringPair("sr25519");
        const ownerAddress = ownerHotkey.address;
        const stakerAddress = stakerHotkey.address;
        const coldkeyAddress = coldkey.address;
        let netuid: number;

        beforeAll(async () => {
            api = context.polkadotJs("Node");

            await forceSetBalance(api, ownerAddress);
            await forceSetBalance(api, stakerAddress);
            await forceSetBalance(api, coldkeyAddress);

            await sudoSetAdminFreezeWindow(api, 0);
            log("Admin freeze window set to 0");

            netuid = await addNewSubnetwork(api, ownerHotkey, coldkey);
            await startCall(api, netuid, coldkey);
            // Set high tempo to prevent emissions during test
            await sudoSetTempo(api, netuid, 10000);
            // Register staker hotkey (not the owner)
            await burnedRegister(api, netuid, stakerAddress, coldkey);
        });

        it({
            id: "T01",
            title: "should remove all stake with price limit",
            test: async () => {
                // Add stake first
                await addStake(api, coldkey, stakerAddress, netuid, tao(100));

                // Get initial stake and balance
                const stakeBefore = await getStake(api, stakerAddress, coldkeyAddress, netuid);
                const balanceBefore = await getBalance(api, coldkeyAddress);
                log(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
                expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

                // Remove all stake with a reasonable limit price (low limit to avoid slippage rejection)
                // Using a low limit price (0.09 TAO per alpha) allows the transaction to succeed
                const limitPrice = tao(1) / 10n; // 0.1 TAO
                await removeStakeFullLimit(api, coldkey, stakerAddress, netuid, limitPrice);

                // Verify stake is zero (staker is not owner, so all stake can be removed)
                const stakeAfter = await getStake(api, stakerAddress, coldkeyAddress, netuid);
                const balanceAfter = await getBalance(api, coldkeyAddress);
                log(`Stake after: ${stakeAfter}, Balance after: ${balanceAfter}`);

                expect(stakeAfter, "Stake should be zero after full removal").toBe(0n);
                expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

                log("✅ Successfully removed all stake with price limit.");
            },
        });

        it({
            id: "T02",
            title: "should remove all stake without price limit",
            test: async () => {
                // Add stake first
                await addStake(api, coldkey, stakerAddress, netuid, tao(100));

                // Get initial stake and balance
                const stakeBefore = await getStake(api, stakerAddress, coldkeyAddress, netuid);
                const balanceBefore = await getBalance(api, coldkeyAddress);
                log(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
                expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

                // Remove all stake without limit price (undefined = no slippage protection)
                await removeStakeFullLimit(api, coldkey, stakerAddress, netuid, undefined);

                // Verify stake is zero (staker is not owner, so all stake can be removed)
                const stakeAfter = await getStake(api, stakerAddress, coldkeyAddress, netuid);
                const balanceAfter = await getBalance(api, coldkeyAddress);
                log(`Stake after: ${stakeAfter}, Balance after: ${balanceAfter}`);

                expect(stakeAfter, "Stake should be zero after full removal").toBe(0n);
                expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

                log("✅ Successfully removed all stake without price limit.");
            },
        });
    },
});
