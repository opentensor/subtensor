import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { generateKeyringPair } from "@moonwall/util";
import {
    addNewSubnetwork,
    addStake,
    burnedRegister,
    forceSetBalance,
    getStake,
    startCall,
    sudoSetTempo,
    tao,
    unstakeAllAlpha,
} from "../../utils";

describeSuite({
    id: "11_unstake_all_alpha",
    title: "▶ unstake_all_alpha extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
        });

        it({
            id: "T01",
            title: "should unstake all alpha from multiple subnets and restake to root",
            test: async () => {
                // Setup accounts
                // - owner1/coldkey: owns subnet 1
                // - owner2/coldkey: owns subnet 2
                // - stakerHotkey: staker (not owner) on both subnets - used for testing unstake_all_alpha
                const owner1Hotkey = generateKeyringPair("sr25519");
                const owner2Hotkey = generateKeyringPair("sr25519");
                const stakerHotkey = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const owner1Address = owner1Hotkey.address;
                const owner2Address = owner2Hotkey.address;
                const stakerAddress = stakerHotkey.address;
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, owner1Address);
                await forceSetBalance(api, owner2Address);
                await forceSetBalance(api, stakerAddress);
                await forceSetBalance(api, coldkeyAddress);

                // Create first subnet with owner1
                const netuid1 = await addNewSubnetwork(api, owner1Hotkey, coldkey);
                await startCall(api, netuid1, coldkey);

                // Create second subnet with owner2
                const netuid2 = await addNewSubnetwork(api, owner2Hotkey, coldkey);
                await startCall(api, netuid2, coldkey);

                // Set very high tempo to prevent emissions during test
                await sudoSetTempo(api, netuid1, 10000);
                await sudoSetTempo(api, netuid2, 10000);

                // Register stakerHotkey on both subnets (it's not the owner)
                await burnedRegister(api, netuid1, stakerAddress, coldkey);
                await burnedRegister(api, netuid2, stakerAddress, coldkey);

                // Add stake to both subnets using stakerHotkey (not the owner)
                await addStake(api, coldkey, stakerAddress, netuid1, tao(100));
                await addStake(api, coldkey, stakerAddress, netuid2, tao(50));

                // Verify stake was added to both subnets
                const stake1Before = await getStake(api, stakerAddress, coldkeyAddress, netuid1);
                const stake2Before = await getStake(api, stakerAddress, coldkeyAddress, netuid2);

                expect(stake1Before, "Should have stake in subnet 1 before unstake_all_alpha").toBeGreaterThan(0n);
                expect(stake2Before, "Should have stake in subnet 2 before unstake_all_alpha").toBeGreaterThan(0n);
                log(`Stake1 before: ${stake1Before}, Stake2 before: ${stake2Before}`);

                // Unstake all alpha - this removes stake from dynamic subnets and restakes to root
                await unstakeAllAlpha(api, coldkey, stakerAddress);

                // Verify stakes are removed from both dynamic subnets
                const stake1After = await getStake(api, stakerAddress, coldkeyAddress, netuid1);
                const stake2After = await getStake(api, stakerAddress, coldkeyAddress, netuid2);

                log(`Stake1 after: ${stake1After}, Stake2 after: ${stake2After}`);

                // Since stakerHotkey is not the owner of either subnet, all stake should be removed
                // High tempo prevents emissions during test, so expect exact zero
                expect(stake1After, "Stake1 should be zero after unstake_all_alpha").toBe(0n);
                expect(stake2After, "Stake2 should be zero after unstake_all_alpha").toBe(0n);

                log("✅ Successfully unstaked all alpha from multiple subnets to root.");
            },
        });
    },
});
