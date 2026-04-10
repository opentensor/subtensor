import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    addNewSubnetwork,
    addStake,
    burnedRegister,
    forceSetBalance,
    generateKeyringPair,
    getStake,
    startCall,
    sudoSetLockReductionInterval,
    swapStake,
    tao,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "07_swap_stake",
    title: "▶ swap_stake extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
        });

        it({
            id: "T01",
            title: "should swap stake from one subnet to another",
            test: async () => {
                // Setup accounts
                const hotkey1 = generateKeyringPair("sr25519");
                const hotkey2 = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const hotkey1Address = hotkey1.address;
                const hotkey2Address = hotkey2.address;
                const coldkeyAddress = coldkey.address;

                await sudoSetLockReductionInterval(api, 1);
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

                // Get initial stakes
                const stake1Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const stake2Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);
                expect(stake1Before, "Should have stake on subnet1 before swap").toBeGreaterThan(0n);

                log(`Stake on netuid1 before: ${stake1Before}, Stake on netuid2 before: ${stake2Before}`);

                // Swap half the stake from subnet1 to subnet2
                // Use raw U64F64 value for the extrinsic
                const stake1 = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const swapAmount = stake1 / 2n;
                await swapStake(api, coldkey, hotkey1Address, netuid1, netuid2, swapAmount);

                // Verify stakes changed
                const stake1After = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
                const stake2After = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);

                log(`Stake on netuid1 after: ${stake1After}, Stake on netuid2 after: ${stake2After}`);

                // Note: hotkey1 is the owner of netuid1, so minimum owner stake may be retained
                expect(stake1After, "Stake on subnet1 should decrease after swap").toBeLessThan(stake1Before);
                expect(stake2After, "Stake on subnet2 should increase after swap").toBeGreaterThan(stake2Before);

                log("✅ Successfully swapped stake from one subnet to another.");
            },
        });
    },
});
