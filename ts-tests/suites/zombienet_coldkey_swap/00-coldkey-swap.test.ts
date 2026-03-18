import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
    ANNOUNCEMENT_DELAY,
    REANNOUNCEMENT_DELAY,
    addNewSubnetwork,
    addStake,
    announceColdkeySwap,
    coldkeyHash,
    disputeColdkeySwap,
    forceSetBalance,
    generateKeyringPair,
    getBalance,
    getHotkeyOwner,
    getOwnedHotkeys,
    getStake,
    getStakingHotkeys,
    getSubnetOwner,
    sendTransaction,
    startCall,
    sudoSetAnnouncementDelay,
    sudoSetReannouncementDelay,
    swapColdkeyAnnounced,
    tao,
    waitForBlocks,
} from "../../utils";

describeSuite({
    id: "00_coldkey_swap",
    title: "▶ coldkey swap extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
            await sudoSetAnnouncementDelay(api, ANNOUNCEMENT_DELAY);
            await sudoSetReannouncementDelay(api, REANNOUNCEMENT_DELAY);
        });

        it({
            id: "T01",
            title: "happy path: announce → wait → swap (verifies full state migration)",
            test: async () => {
                const oldColdkey = generateKeyringPair("sr25519");
                const newColdkey = generateKeyringPair("sr25519");
                const hotkey = generateKeyringPair("sr25519");

                await forceSetBalance(api, oldColdkey.address);
                await forceSetBalance(api, hotkey.address);

                // Create a subnet (oldColdkey becomes subnet owner)
                const netuid = await addNewSubnetwork(api, hotkey, oldColdkey);
                await startCall(api, netuid, oldColdkey);
                log(`Created subnet ${netuid}`);

                // Add stake
                await addStake(api, oldColdkey, hotkey.address, netuid, tao(200));

                // Snapshot state before swap
                const stakeBefore = await getStake(api, hotkey.address, oldColdkey.address, netuid);
                expect(stakeBefore, "should have stake before swap").toBeGreaterThan(0n);
                expect(await getSubnetOwner(api, netuid), "old coldkey should own the subnet").toBe(oldColdkey.address);
                expect(await getHotkeyOwner(api, hotkey.address), "old coldkey should own the hotkey").toBe(oldColdkey.address);
                expect(await getOwnedHotkeys(api, oldColdkey.address), "old coldkey should have owned hotkeys").toContain(hotkey.address);
                expect(await getStakingHotkeys(api, oldColdkey.address), "old coldkey should have staking hotkeys").toContain(hotkey.address);
                const balanceBefore = await getBalance(api, oldColdkey.address);
                log(`Before swap — stake: ${stakeBefore}, balance: ${balanceBefore}`);

                // Announce
                const announceResult = await sendTransaction(
                    api,
                    api.tx.subtensorModule.announceColdkeySwap(coldkeyHash(newColdkey)),
                    oldColdkey,
                );
                expect(announceResult.success, "announce should succeed").toBe(true);
                const announcedEvent = announceResult.events.find((e) => e.method === "ColdkeySwapAnnounced");
                expect(announcedEvent, "ColdkeySwapAnnounced event should be emitted").toBeDefined();
                log("Announced coldkey swap");

                // Wait for delay
                await waitForBlocks(api, ANNOUNCEMENT_DELAY + 1);

                // Swap
                const swapResult = await sendTransaction(
                    api,
                    api.tx.subtensorModule.swapColdkeyAnnounced(newColdkey.address),
                    oldColdkey,
                );
                expect(swapResult.success, "swap should succeed").toBe(true);
                const swappedEvent = swapResult.events.find((e) => e.method === "ColdkeySwapped");
                expect(swappedEvent, "ColdkeySwapped event should be emitted").toBeDefined();
                log("Swap executed");

                // Verify stake migrated
                const stakeOldAfter = await getStake(api, hotkey.address, oldColdkey.address, netuid);
                expect(stakeOldAfter, "old coldkey should have no stake").toBe(0n);
                const stakeNewAfter = await getStake(api, hotkey.address, newColdkey.address, netuid);
                expect(stakeNewAfter, "new coldkey should have the stake").toBeGreaterThan(0n);
                log(`Stake migrated: old=${stakeOldAfter}, new=${stakeNewAfter}`);

                // Verify subnet ownership transferred
                expect(await getSubnetOwner(api, netuid), "new coldkey should own the subnet").toBe(newColdkey.address);
                log("Subnet ownership transferred");

                // Verify hotkey ownership transferred
                expect(await getHotkeyOwner(api, hotkey.address), "new coldkey should own the hotkey").toBe(newColdkey.address);
                expect(await getOwnedHotkeys(api, oldColdkey.address), "old coldkey should have no owned hotkeys").not.toContain(hotkey.address);
                expect(await getOwnedHotkeys(api, newColdkey.address), "new coldkey should own the hotkey").toContain(hotkey.address);
                log("Hotkey ownership transferred");

                // Verify staking hotkeys transferred
                expect(await getStakingHotkeys(api, oldColdkey.address), "old coldkey should have no staking hotkeys").not.toContain(hotkey.address);
                expect(await getStakingHotkeys(api, newColdkey.address), "new coldkey should have staking hotkeys").toContain(hotkey.address);
                log("Staking hotkeys transferred");

                // Verify balance transferred
                const balanceOldAfter = await getBalance(api, oldColdkey.address);
                expect(balanceOldAfter, "old coldkey balance should be 0").toBe(0n);
                const balanceNewAfter = await getBalance(api, newColdkey.address);
                expect(balanceNewAfter, "new coldkey should have balance").toBeGreaterThan(0n);
                log(`Balance: old=${balanceOldAfter}, new=${balanceNewAfter}`);
            },
        });

        it({
            id: "T02",
            title: "swap too early: rejected",
            test: async () => {
                const oldColdkey = generateKeyringPair("sr25519");
                const newColdkey = generateKeyringPair("sr25519");
                await forceSetBalance(api, oldColdkey.address);

                await announceColdkeySwap(api, oldColdkey, coldkeyHash(newColdkey));

                // Immediately try swap without waiting
                const result = await sendTransaction(
                    api,
                    api.tx.subtensorModule.swapColdkeyAnnounced(newColdkey.address),
                    oldColdkey,
                );
                expect(result.success, "swap should be rejected (too early)").toBe(false);
                log("Correctly rejected early swap");
            },
        });

        it({
            id: "T03",
            title: "reannouncement: too early → wait → reannounce → swap",
            test: async () => {
                const oldColdkey = generateKeyringPair("sr25519");
                const newColdkey1 = generateKeyringPair("sr25519");
                const newColdkey2 = generateKeyringPair("sr25519");
                await forceSetBalance(api, oldColdkey.address);

                // First announcement
                await announceColdkeySwap(api, oldColdkey, coldkeyHash(newColdkey1));
                log("First announce ok");

                // Reannounce immediately (should fail)
                const earlyResult = await sendTransaction(
                    api,
                    api.tx.subtensorModule.announceColdkeySwap(coldkeyHash(newColdkey2)),
                    oldColdkey,
                );
                expect(earlyResult.success, "early reannounce should fail").toBe(false);
                log("Early reannounce rejected");

                // Wait for reannouncement delay
                await waitForBlocks(api, REANNOUNCEMENT_DELAY + 1);

                // Reannounce (should succeed)
                await announceColdkeySwap(api, oldColdkey, coldkeyHash(newColdkey2));
                log("Reannounced to new key");

                // Wait for announcement delay
                await waitForBlocks(api, ANNOUNCEMENT_DELAY + 1);

                // Swap with old hash (should fail)
                const wrongResult = await sendTransaction(
                    api,
                    api.tx.subtensorModule.swapColdkeyAnnounced(newColdkey1.address),
                    oldColdkey,
                );
                expect(wrongResult.success, "swap with old hash should fail").toBe(false);
                log("Old hash rejected");

                // Swap with new hash (should succeed)
                await swapColdkeyAnnounced(api, oldColdkey, newColdkey2.address);
                log("Swap with reannounced key succeeded");
            },
        });

        it({
            id: "T04",
            title: "dispute blocks swap execution",
            test: async () => {
                const oldColdkey = generateKeyringPair("sr25519");
                const newColdkey = generateKeyringPair("sr25519");
                await forceSetBalance(api, oldColdkey.address);

                await announceColdkeySwap(api, oldColdkey, coldkeyHash(newColdkey));

                // Dispute
                const disputeResult = await sendTransaction(
                    api,
                    api.tx.subtensorModule.disputeColdkeySwap(),
                    oldColdkey,
                );
                expect(disputeResult.success, "dispute should succeed").toBe(true);
                const disputeEvent = disputeResult.events.find((e) => e.method === "ColdkeySwapDisputed");
                expect(disputeEvent, "ColdkeySwapDisputed event should be emitted").toBeDefined();
                log("Disputed");

                await waitForBlocks(api, ANNOUNCEMENT_DELAY + 1);

                // Swap should fail (disputed)
                const swapResult = await sendTransaction(
                    api,
                    api.tx.subtensorModule.swapColdkeyAnnounced(newColdkey.address),
                    oldColdkey,
                );
                expect(swapResult.success, "swap should fail (disputed)").toBe(false);
                log("Swap blocked after dispute");
            },
        });

        it({
            id: "T05",
            title: "double dispute: second fails",
            test: async () => {
                const oldColdkey = generateKeyringPair("sr25519");
                const newColdkey = generateKeyringPair("sr25519");
                await forceSetBalance(api, oldColdkey.address);

                await announceColdkeySwap(api, oldColdkey, coldkeyHash(newColdkey));
                await disputeColdkeySwap(api, oldColdkey);

                // Second dispute should fail
                const result = await sendTransaction(
                    api,
                    api.tx.subtensorModule.disputeColdkeySwap(),
                    oldColdkey,
                );
                expect(result.success, "second dispute should fail").toBe(false);
                log("Second dispute correctly rejected");
            },
        });

        it({
            id: "T06",
            title: "announce fails: insufficient balance",
            test: async () => {
                const poorKey = generateKeyringPair("sr25519");
                // Intentionally NOT funded

                const result = await sendTransaction(
                    api,
                    api.tx.subtensorModule.announceColdkeySwap(
                        coldkeyHash(generateKeyringPair("sr25519")),
                    ),
                    poorKey,
                );
                expect(result.success, "announce should fail (no balance)").toBe(false);
                log("Announce rejected for insufficient balance");
            },
        });

        it({
            id: "T07",
            title: "swap with wrong key: hash mismatch",
            test: async () => {
                const oldColdkey = generateKeyringPair("sr25519");
                const newColdkey = generateKeyringPair("sr25519");
                const wrongKey = generateKeyringPair("sr25519");
                await forceSetBalance(api, oldColdkey.address);

                await announceColdkeySwap(api, oldColdkey, coldkeyHash(newColdkey));
                await waitForBlocks(api, ANNOUNCEMENT_DELAY + 1);

                // Swap with wrong address
                const result = await sendTransaction(
                    api,
                    api.tx.subtensorModule.swapColdkeyAnnounced(wrongKey.address),
                    oldColdkey,
                );
                expect(result.success, "swap should fail (hash mismatch)").toBe(false);
                log("Hash mismatch correctly rejected");
            },
        });

        it({
            id: "T08",
            title: "dispute without announcement: fails",
            test: async () => {
                const someKey = generateKeyringPair("sr25519");
                await forceSetBalance(api, someKey.address);

                const result = await sendTransaction(
                    api,
                    api.tx.subtensorModule.disputeColdkeySwap(),
                    someKey,
                );
                expect(result.success, "dispute should fail (no announcement)").toBe(false);
                log("Dispute without announcement rejected");
            },
        });
    },
});
