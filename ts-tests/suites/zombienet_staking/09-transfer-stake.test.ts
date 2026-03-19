import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import {
    addNewSubnetwork,
    addStake,
    forceSetBalance,
    generateKeyringPair,
    getStake,
    startCall,
    sudoSetLockReductionInterval,
    tao,
    transferStake,
} from "../../utils";
import { subtensor } from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";

describeSuite({
    id: "09_transfer_stake",
    title: "▶ transfer_stake extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: TypedApi<typeof subtensor>;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);
        });

        it({
            id: "T01",
            title: "should transfer stake to another coldkey across subnets",
            test: async () => {
                // Setup accounts
                const hotkey1 = generateKeyringPair("sr25519");
                const hotkey2 = generateKeyringPair("sr25519");
                const originColdkey = generateKeyringPair("sr25519");
                const destinationColdkey = generateKeyringPair("sr25519");
                const hotkey1Address = hotkey1.address;
                const hotkey2Address = hotkey2.address;
                const originColdkeyAddress = originColdkey.address;
                const destinationColdkeyAddress = destinationColdkey.address;

                await forceSetBalance(api, hotkey1Address);
                await forceSetBalance(api, hotkey2Address);
                await forceSetBalance(api, originColdkeyAddress);
                await forceSetBalance(api, destinationColdkeyAddress);

                await sudoSetLockReductionInterval(api, 1);
                // Create first subnet
                const netuid1 = await addNewSubnetwork(api, hotkey1, originColdkey);
                await startCall(api, netuid1, originColdkey);

                // Create second subnet
                const netuid2 = await addNewSubnetwork(api, hotkey2, originColdkey);
                await startCall(api, netuid2, originColdkey);

                // Add stake from origin coldkey on first subnet
                await addStake(api, originColdkey, hotkey1Address, netuid1, tao(200));

                // Get initial stakes (converted from U64F64 for display)
                const originStakeBefore = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
                const destStakeBefore = await getStake(api, hotkey1Address, destinationColdkeyAddress, netuid2);
                expect(originStakeBefore, "Origin should have stake before transfer").toBeGreaterThan(0n);

                log(
                    `Origin stake (netuid1) before: ${originStakeBefore}, Destination stake (netuid2) before: ${destStakeBefore}`
                );

                // Transfer stake to destination coldkey on a different subnet
                const originStake = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
                const transferAmount = originStake / 2n;
                await transferStake(
                    api,
                    originColdkey,
                    destinationColdkeyAddress,
                    hotkey1Address,
                    netuid1,
                    netuid2,
                    transferAmount
                );

                // Verify stakes changed
                const originStakeAfter = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
                const destStakeAfter = await getStake(api, hotkey1Address, destinationColdkeyAddress, netuid2);

                log(
                    `Origin stake (netuid1) after: ${originStakeAfter}, Destination stake (netuid2) after: ${destStakeAfter}`
                );

                expect(originStakeAfter, "Origin stake should decrease").toBeLessThan(originStakeBefore);
                expect(destStakeAfter, "Destination stake should increase").toBeGreaterThan(destStakeBefore);

                log("✅ Successfully transferred stake to another coldkey across subnets.");
            },
        });

        it({
            id: "T02",
            title: "",
            test: async () => {
                // Setup accounts
                const hotkey = generateKeyringPair("sr25519");
                const originColdkey = generateKeyringPair("sr25519");
                const destinationColdkey = generateKeyringPair("sr25519");
                const hotkeyAddress = hotkey.address;
                const originColdkeyAddress = originColdkey.address;
                const destinationColdkeyAddress = destinationColdkey.address;

                await forceSetBalance(api, hotkeyAddress);
                await forceSetBalance(api, originColdkeyAddress);
                await forceSetBalance(api, destinationColdkeyAddress);

                // Create subnet
                const netuid = await addNewSubnetwork(api, hotkey, originColdkey);
                await startCall(api, netuid, originColdkey);

                // Add stake from origin coldkey
                const stakeAmount = tao(100);
                await addStake(api, originColdkey, hotkeyAddress, netuid, stakeAmount);

                // Get initial stake (converted from U64F64 for display)
                const originStakeBefore = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
                expect(originStakeBefore, "Origin should have stake before transfer").toBeGreaterThan(0n);

                log(`Origin stake before: ${originStakeBefore}`);

                // Transfer stake to destination coldkey
                const originStake = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
                const transferAmount = originStake / 2n;
                await transferStake(
                    api,
                    originColdkey,
                    destinationColdkeyAddress,
                    hotkeyAddress,
                    netuid,
                    netuid,
                    transferAmount
                );

                // Verify destination received stake
                const originStakeAfter = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
                const destStakeAfter = await getStake(api, hotkeyAddress, destinationColdkeyAddress, netuid);

                log(`Origin stake after: ${originStakeAfter}, Destination stake after: ${destStakeAfter}`);

                expect(originStakeAfter, "Origin stake should decrease after transfer").toBeLessThan(originStakeBefore);
                expect(destStakeAfter, "Destination stake should be non-zero after transfer").toBeGreaterThan(0n);

                log("✅ Successfully transferred stake to another coldkey.");
            },
        });
    },
});
