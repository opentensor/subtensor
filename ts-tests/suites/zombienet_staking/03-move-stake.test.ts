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
    moveStake,
    startCall,
    tao,
} from "../../utils";

describeSuite({
    id: "03_move_stake",
    title: "▶ move_stake extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
        });

        it({
            id: "T01",
            title: "should move stake to another hotkey across subnets",
            test: async () => {
                // Setup accounts
                const originHotkey = generateKeyringPair("sr25519");
                const destinationHotkey = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const originHotkeyAddress = originHotkey.address;
                const destinationHotkeyAddress = destinationHotkey.address;
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, originHotkeyAddress);
                await forceSetBalance(api, destinationHotkeyAddress);
                await forceSetBalance(api, coldkeyAddress);

                // Create first subnet with origin hotkey
                const netuid1 = await addNewSubnetwork(api, originHotkey, coldkey);
                await startCall(api, netuid1, coldkey);

                // Create second subnet with destination hotkey
                const netuid2 = await addNewSubnetwork(api, destinationHotkey, coldkey);
                await startCall(api, netuid2, coldkey);

                // Add stake to origin hotkey on first subnet
                await addStake(api, coldkey, originHotkeyAddress, netuid1, tao(200));

                // Get initial stakes (converted from U64F64 for display)
                const originStakeBefore = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid1);
                const destStakeBefore = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid2);
                expect(originStakeBefore, "Origin hotkey should have stake before move").toBeGreaterThan(0n);

                log(
                    `Origin stake (netuid1) before: ${originStakeBefore}, Destination stake (netuid2) before: ${destStakeBefore}`
                );

                // Move stake to destination hotkey on different subnet
                // Use raw U64F64 value for the extrinsic
                const originStake = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid1);
                const moveAmount = originStake / 2n;
                await moveStake(
                    api,
                    coldkey,
                    originHotkeyAddress,
                    destinationHotkeyAddress,
                    netuid1,
                    netuid2,
                    moveAmount
                );

                // Verify stakes changed
                const originStakeAfter = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid1);
                const destStakeAfter = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid2);

                log(
                    `Origin stake (netuid1) after: ${originStakeAfter}, Destination stake (netuid2) after: ${destStakeAfter}`
                );

                expect(originStakeAfter, "Origin stake should decrease").toBeLessThan(originStakeBefore);
                expect(destStakeAfter, "Destination stake should increase").toBeGreaterThan(destStakeBefore);

                log("✅ Successfully moved stake to another hotkey across subnets.");
            },
        });

        it({
            id: "T02",
            title: "should move stake to another hotkey on the same subnet",
            test: async () => {
                // Setup accounts
                const originHotkey = generateKeyringPair("sr25519");
                const destinationHotkey = generateKeyringPair("sr25519");
                const coldkey = generateKeyringPair("sr25519");
                const originHotkeyAddress = originHotkey.address;
                const destinationHotkeyAddress = destinationHotkey.address;
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, originHotkeyAddress);
                await forceSetBalance(api, destinationHotkeyAddress);
                await forceSetBalance(api, coldkeyAddress);

                // Create subnet with origin hotkey
                const netuid = await addNewSubnetwork(api, originHotkey, coldkey);
                await startCall(api, netuid, coldkey);

                // Register destination hotkey on the same subnet
                await burnedRegister(api, netuid, destinationHotkeyAddress, coldkey);

                // Add stake to origin hotkey
                await addStake(api, coldkey, originHotkeyAddress, netuid, tao(200));

                // Get initial stakes (converted from U64F64 for display)
                const originStakeBefore = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid);
                const destStakeBefore = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid);
                expect(originStakeBefore, "Origin hotkey should have stake before move").toBeGreaterThan(0n);

                log(`Origin stake before: ${originStakeBefore}, Destination stake before: ${destStakeBefore}`);

                // Move stake to destination hotkey on the same subnet
                // Use raw U64F64 value for the extrinsic
                const originStake = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid);
                const moveAmount = originStake / 2n;
                await moveStake(
                    api,
                    coldkey,
                    originHotkeyAddress,
                    destinationHotkeyAddress,
                    netuid,
                    netuid,
                    moveAmount
                );

                // Verify stakes changed
                const originStakeAfter = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid);
                const destStakeAfter = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid);

                log(`Origin stake after: ${originStakeAfter}, Destination stake after: ${destStakeAfter}`);

                expect(originStakeAfter, "Origin stake should decrease").toBeLessThan(originStakeBefore);
                expect(destStakeAfter, "Destination stake should increase").toBeGreaterThan(destStakeBefore);

                log("✅ Successfully moved stake to another hotkey on the same subnet.");
            },
        });
    },
});
