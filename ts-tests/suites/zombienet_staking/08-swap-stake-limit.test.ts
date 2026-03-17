import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
    addNewSubnetwork,
    addStake,
    burnedRegister,
    forceSetBalance,
    generateKeyringPair,
    getStake,
    startCall,
    swapStakeLimit,
    tao,
} from "../../utils";

describeSuite({
    id: "08_swap_stake_limit",
    title: "▶ swap_stake_limit extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
        });

        it({
            id: "T01",
            title: "should swap stake with price limit (allow partial)",
            test: async () => {
                // Setup accounts
                const hotkey1 = generateKeyringPair("sr25519");
                const hotkey2 = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const hotkey1Address = hotkey1.address;
                const hotkey2Address = hotkey2.address;
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, hotkey1Address);
                await forceSetBalance(api, hotkey2Address);
                await forceSetBalance(api, coldkeyAddress);

                // Create first subnet
                const netuid1 = await addNewSubnetwork(api, hotkey1, coldkey);
                await startCall(api, netuid1, coldkey);

                // Create second subnet
                const netuid2 = await addNewSubnetwork(api, hotkey2, coldkey);
                await startCall(api, netuid2, coldkey);

                // Register hotkey1 on subnet2 so we can swap stake there
                await burnedRegister(api, netuid2, hotkey1Address, coldkey);

                // Add stake to hotkey1 on subnet1
                await addStake(api, coldkey, hotkey1Address, netuid1, tao(100));

                // Get initial stakes (converted from U64F64 for display)
                const stake1Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const stake2Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);
                expect(stake1Before, "Should have stake on subnet1 before swap").toBeGreaterThan(0n);

                log(`Stake on netuid1 before: ${stake1Before}, Stake on netuid2 before: ${stake2Before}`);

                // Swap stake with limit price (0.99 TAO relative price limit, allow partial fills)
                const stake1 = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const swapAmount = stake1 / 2n;
                const limitPrice = (tao(1) * 99n) / 100n; // 0.99 TAO
                await swapStakeLimit(api, coldkey, hotkey1Address, netuid1, netuid2, swapAmount, limitPrice, true);

                // Verify stakes changed
                const stake1After = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const stake2After = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);

                log(`Stake on netuid1 after: ${stake1After}, Stake on netuid2 after: ${stake2After}`);

                expect(stake1After, "Stake on subnet1 should decrease").toBeLessThan(stake1Before);
                expect(stake2After, "Stake on subnet2 should increase").toBeGreaterThan(stake2Before);

                log("✅ Successfully swapped stake with price limit (allow partial).");
            },
        });

        it({
            id: "T02",
            title: "should swap stake with price limit (fill or kill)",
            test: async () => {
                // Setup accounts
                const hotkey1 = generateKeyringPair("sr25519");
                const hotkey2 = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const hotkey1Address = hotkey1.address;
                const hotkey2Address = hotkey2.address;
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, hotkey1Address);
                await forceSetBalance(api, hotkey2Address);
                await forceSetBalance(api, coldkeyAddress);

                // Create first subnet
                const netuid1 = await addNewSubnetwork(api, hotkey1, coldkey);
                await startCall(api, netuid1, coldkey);

                // Create second subnet
                const netuid2 = await addNewSubnetwork(api, hotkey2, coldkey);
                await startCall(api, netuid2, coldkey);

                // Register hotkey1 on subnet2 so we can swap stake there
                await burnedRegister(api, netuid2, hotkey1Address, coldkey);

                // Add stake to hotkey1 on subnet1
                await addStake(api, coldkey, hotkey1Address, netuid1, tao(100));

                // Get initial stakes (converted from U64F64 for display)
                const stake1Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const stake2Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);
                expect(stake1Before, "Should have stake on subnet1 before swap").toBeGreaterThan(0n);

                log(`Stake on netuid1 before: ${stake1Before}, Stake on netuid2 before: ${stake2Before}`);

                // Swap stake with limit price (fill or kill mode - allow_partial = false)
                const stake1 = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const swapAmount = stake1 / 2n;
                const limitPrice = tao(1) / 10n; // 0.1 TAO - permissive limit to allow slippage
                await swapStakeLimit(api, coldkey, hotkey1Address, netuid1, netuid2, swapAmount, limitPrice, false);

                // Verify stakes changed
                const stake1After = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const stake2After = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);

                log(`Stake on netuid1 after: ${stake1After}, Stake on netuid2 after: ${stake2After}`);

                expect(stake1After, "Stake on subnet1 should decrease").toBeLessThan(stake1Before);
                expect(stake2After, "Stake on subnet2 should increase").toBeGreaterThan(stake2Before);

                log("✅ Successfully swapped stake with price limit (fill or kill).");
            },
        });
    },
});
