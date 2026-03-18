import { expect, beforeAll } from "vitest";
import { describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
    addNewSubnetwork,
    addStake,
    claimRoot,
    forceSetBalance,
    generateKeyringPair,
    getPendingRootAlphaDivs,
    getRootClaimable,
    getRootClaimed,
    getRootClaimType,
    getStake,
    getSubnetAlphaIn,
    getSubnetMovingPrice,
    getSubnetTAO,
    getTaoWeight,
    getTotalHotkeyAlpha,
    isSubtokenEnabled,
    setRootClaimType,
    startCall,
    sudoSetAdminFreezeWindow,
    sudoSetEmaPriceHalvingPeriod,
    sudoSetLockReductionInterval,
    sudoSetRootClaimThreshold,
    sudoSetSubnetMovingAlpha,
    sudoSetSubtokenEnabled,
    sudoSetTempo,
    tao,
    waitForBlocks,
} from "../../utils";

describeSuite({
    id: "0203_claim_root",
    title: "▶ claim_root extrinsic",
    foundationMethods: "zombie",
    testCases: ({ it, context, log }) => {
        let api: ApiPromise;
        const ROOT_NETUID = 0;

        beforeAll(async () => {
            api = context.polkadotJs("Node");
            await sudoSetLockReductionInterval(api, 1);
        });

        it({
            id: "T0401",
            title: "should claim root dividends with Keep type (stake to dynamic subnet)",
            test: async () => {
                // Setup accounts
                // - owner1Hotkey/owner1Coldkey: subnet 1 owner
                // - owner2Hotkey/owner2Coldkey: subnet 2 owner (needed for root_sell_flag)
                // - stakerColdkey: the coldkey that will stake on root and claim dividends
                const owner1Hotkey = generateKeyringPair("sr25519");
                const owner1Coldkey = generateKeyringPair("sr25519");
                const owner2Hotkey = generateKeyringPair("sr25519");
                const owner2Coldkey = generateKeyringPair("sr25519");
                const stakerColdkey = generateKeyringPair("sr25519");
                const owner1HotkeyAddress = owner1Hotkey.address;
                const owner1ColdkeyAddress = owner1Coldkey.address;
                const owner2HotkeyAddress = owner2Hotkey.address;
                const owner2ColdkeyAddress = owner2Coldkey.address;
                const stakerColdkeyAddress = stakerColdkey.address;

                // Fund all accounts
                await forceSetBalance(api, owner1HotkeyAddress);
                await forceSetBalance(api, owner1ColdkeyAddress);
                await forceSetBalance(api, owner2HotkeyAddress);
                await forceSetBalance(api, owner2ColdkeyAddress);
                await forceSetBalance(api, stakerColdkeyAddress);

                // Disable admin freeze window to allow enabling subtoken for ROOT
                await sudoSetAdminFreezeWindow(api, 0);
                log("Admin freeze window set to 0");

                // Enable subtoken for ROOT subnet (required for staking on root)
                const subtokenEnabledBefore = await isSubtokenEnabled(api, ROOT_NETUID);
                if (!subtokenEnabledBefore) {
                    await sudoSetSubtokenEnabled(api, ROOT_NETUID, "Yes");
                    const subtokenEnabledAfter = await isSubtokenEnabled(api, ROOT_NETUID);
                    log(`ROOT subtoken enabled: ${subtokenEnabledAfter}`);
                    expect(subtokenEnabledAfter).toBe(true);
                }

                // Create TWO dynamic subnets - needed for root_sell_flag to become true
                // root_sell_flag = sum(moving_prices) > 1.0
                // Each subnet's moving price approaches 1.0 via EMA, so 2 subnets can exceed threshold
                const netuid1 = await addNewSubnetwork(api, owner1Hotkey, owner1Coldkey);
                await startCall(api, netuid1, owner1Coldkey);
                log(`Created subnet 1 with netuid: ${netuid1}`);

                const netuid2 = await addNewSubnetwork(api, owner2Hotkey, owner2Coldkey);
                await startCall(api, netuid2, owner2Coldkey);
                log(`Created subnet 2 with netuid: ${netuid2}`);

                // Set short tempo for faster emission distribution
                await sudoSetTempo(api, netuid1, 1);
                await sudoSetTempo(api, netuid2, 1);
                log("Set tempo to 1 for both subnets");

                // Set EMA price halving period to 1 for fast moving price convergence
                // Formula: alpha = SubnetMovingAlpha * blocks/(blocks + halving_time)
                // With halving_time=1: after 10 blocks, alpha ≈ 0.91, moving price ≈ 0.91
                // With 2 subnets at ~0.9 each, total > 1.0 enabling root_sell_flag
                await sudoSetEmaPriceHalvingPeriod(api, netuid1, 1);
                await sudoSetEmaPriceHalvingPeriod(api, netuid2, 1);
                log("Set EMA halving period to 1 for fast price convergence");

                // Set SubnetMovingAlpha to 1.0 (default is 0.000003 which is way too slow)
                // I96F32 encoding: 1.0 * 2^32 = 4294967296
                const movingAlpha = BigInt(4294967296); // 1.0 in I96F32
                await sudoSetSubnetMovingAlpha(api, movingAlpha);
                log("Set SubnetMovingAlpha to 1.0 for fast EMA convergence");

                // Set threshold to 0 to allow claiming any amount
                await sudoSetRootClaimThreshold(api, netuid1, 0n);
                await sudoSetRootClaimThreshold(api, netuid2, 0n);

                // Add stake to ROOT subnet for the staker (makes them eligible for root dividends)
                const rootStakeAmount = tao(100);
                await addStake(api, stakerColdkey, owner1HotkeyAddress, ROOT_NETUID, rootStakeAmount);
                log(`Added ${rootStakeAmount} stake to root subnet for staker`);

                // Verify root stake was added
                const rootStake = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, ROOT_NETUID);
                log(`Root stake: ${rootStake}`);
                expect(rootStake, "Should have stake on root subnet").toBeGreaterThan(0n);

                // Add stake to both dynamic subnets (owner stake to enable emissions flow)
                const subnetStakeAmount = tao(50);
                await addStake(api, owner1Coldkey, owner1HotkeyAddress, netuid1, subnetStakeAmount);
                await addStake(api, owner2Coldkey, owner2HotkeyAddress, netuid2, subnetStakeAmount);
                log(`Added ${subnetStakeAmount} owner stake to subnets ${netuid1} and ${netuid2}`);

                // Get initial stake on subnet 1 for the staker (should be 0)
                const stakerSubnetStakeBefore = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, netuid1);
                log(`Staker subnet stake before claim: ${stakerSubnetStakeBefore}`);

                // Set root claim type to Keep (keep alpha on subnet instead of swapping to TAO)
                await setRootClaimType(api, stakerColdkey, "Keep");
                const claimType = await getRootClaimType(api, stakerColdkeyAddress);
                log(`Root claim type: ${claimType}`);
                expect(claimType).toBe("Keep");

                // Wait for blocks to:
                // 1. Allow moving prices to converge (need sum > 1.0 for root_sell_flag)
                // 2. Accumulate PendingRootAlphaDivs
                // 3. Distribute emissions at tempo boundary
                const blocksToWait = 25;
                log(`Waiting for ${blocksToWait} blocks for moving prices to converge and emissions to accumulate...`);
                await waitForBlocks(api, blocksToWait);

                // Debug: Check key storage values
                const subnetTaoRoot = await getSubnetTAO(api, ROOT_NETUID);
                const subnetTao1 = await getSubnetTAO(api, netuid1);
                const subnetTao2 = await getSubnetTAO(api, netuid2);
                log(`SubnetTAO - ROOT: ${subnetTaoRoot}, netuid1: ${subnetTao1}, netuid2: ${subnetTao2}`);

                const movingPrice1 = await getSubnetMovingPrice(api, netuid1);
                const movingPrice2 = await getSubnetMovingPrice(api, netuid2);
                log(`SubnetMovingPrice - netuid1: ${movingPrice1}, netuid2: ${movingPrice2}`);
                // Note: Moving price is I96F32, so divide by 2^32 to get actual value
                const mp1Float = Number(movingPrice1) / 2 ** 32;
                const mp2Float = Number(movingPrice2) / 2 ** 32;
                log(
                    `SubnetMovingPrice (float) - netuid1: ${mp1Float}, netuid2: ${mp2Float}, sum: ${mp1Float + mp2Float}`
                );

                const pendingDivs1 = await getPendingRootAlphaDivs(api, netuid1);
                const pendingDivs2 = await getPendingRootAlphaDivs(api, netuid2);
                log(`PendingRootAlphaDivs - netuid1: ${pendingDivs1}, netuid2: ${pendingDivs2}`);

                const taoWeight = await getTaoWeight(api);
                log(`TaoWeight: ${taoWeight}`);

                const alphaIn1 = await getSubnetAlphaIn(api, netuid1);
                const alphaIn2 = await getSubnetAlphaIn(api, netuid2);
                log(`SubnetAlphaIn - netuid1: ${alphaIn1}, netuid2: ${alphaIn2}`);

                const totalHotkeyAlpha1 = await getTotalHotkeyAlpha(api, owner1HotkeyAddress, netuid1);
                log(`TotalHotkeyAlpha for hotkey1 on netuid1: ${totalHotkeyAlpha1}`);

                // Check if there are any claimable dividends
                const claimable = await getRootClaimable(api, owner1HotkeyAddress);
                const claimableStr = [...claimable.entries()].map(([k, v]) => `[${k}: ${v.toString()}]`).join(", ");
                log(`RootClaimable entries for hotkey1: ${claimableStr || "(none)"}`);

                // Call claim_root to claim dividends for subnet 1
                await claimRoot(api, stakerColdkey, [netuid1]);
                log("Called claim_root");

                // Get stake on subnet 1 after claim
                const stakerSubnetStakeAfter = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, netuid1);
                log(`Staker subnet stake after claim: ${stakerSubnetStakeAfter}`);

                // Check RootClaimed value
                const rootClaimed = await getRootClaimed(api, netuid1, owner1HotkeyAddress, stakerColdkeyAddress);
                log(`RootClaimed value: ${rootClaimed}`);

                // Verify dividends were claimed
                expect(stakerSubnetStakeAfter, "Stake should increase after claiming root dividends").toBeGreaterThan(
                    stakerSubnetStakeBefore
                );
                log(
                    `✅ Root claim successful: stake increased from ${stakerSubnetStakeBefore} to ${stakerSubnetStakeAfter}`
                );
            },
        });

        it({
            id: "T0402",
            title: "should claim root dividends with Swap type (swap to TAO on ROOT)",
            test: async () => {
                // Setup accounts
                // - owner1Hotkey/owner1Coldkey: subnet 1 owner
                // - owner2Hotkey/owner2Coldkey: subnet 2 owner (needed for root_sell_flag)
                // - stakerColdkey: the coldkey that will stake on root and claim dividends
                const owner1Hotkey = generateKeyringPair("sr25519");
                const owner1Coldkey = generateKeyringPair("sr25519");
                const owner2Hotkey = generateKeyringPair("sr25519");
                const owner2Coldkey = generateKeyringPair("sr25519");
                const stakerColdkey = generateKeyringPair("sr25519");
                const owner1HotkeyAddress = owner1Hotkey.address;
                const owner1ColdkeyAddress = owner1Coldkey.address;
                const owner2HotkeyAddress = owner2Hotkey.address;
                const owner2ColdkeyAddress = owner2Coldkey.address;
                const stakerColdkeyAddress = stakerColdkey.address;

                // Fund all accounts
                await forceSetBalance(api, owner1HotkeyAddress);
                await forceSetBalance(api, owner1ColdkeyAddress);
                await forceSetBalance(api, owner2HotkeyAddress);
                await forceSetBalance(api, owner2ColdkeyAddress);
                await forceSetBalance(api, stakerColdkeyAddress);

                // Disable admin freeze window to allow enabling subtoken for ROOT
                await sudoSetAdminFreezeWindow(api, 0);
                log("Admin freeze window set to 0");

                // Create TWO dynamic subnets
                const netuid1 = await addNewSubnetwork(api, owner1Hotkey, owner1Coldkey);
                await startCall(api, netuid1, owner1Coldkey);
                log(`Created subnet 1 with netuid: ${netuid1}`);

                const netuid2 = await addNewSubnetwork(api, owner2Hotkey, owner2Coldkey);
                await startCall(api, netuid2, owner2Coldkey);
                log(`Created subnet 2 with netuid: ${netuid2}`);

                // Set short tempo for faster emission distribution
                await sudoSetTempo(api, netuid1, 1);
                await sudoSetTempo(api, netuid2, 1);
                log("Set tempo to 1 for both subnets");

                // Set EMA price halving period to 1 for fast moving price convergence
                await sudoSetEmaPriceHalvingPeriod(api, netuid1, 1);
                await sudoSetEmaPriceHalvingPeriod(api, netuid2, 1);
                log("Set EMA halving period to 1 for fast price convergence");

                // Set SubnetMovingAlpha to 1.0 (default is 0.000003 which is way too slow)
                // I96F32 encoding: 1.0 * 2^32 = 4294967296
                const movingAlpha = BigInt(4294967296); // 1.0 in I96F32
                await sudoSetSubnetMovingAlpha(api, movingAlpha);
                log("Set SubnetMovingAlpha to 1.0 for fast EMA convergence");

                // Set threshold to 0 to allow claiming any amount
                await sudoSetRootClaimThreshold(api, netuid1, 0n);
                await sudoSetRootClaimThreshold(api, netuid2, 0n);

                // Add stake to ROOT subnet for the staker
                const rootStakeAmount = tao(100);
                await addStake(api, stakerColdkey, owner1HotkeyAddress, ROOT_NETUID, rootStakeAmount);
                log(`Added ${rootStakeAmount} stake to root subnet for staker`);

                // Get initial ROOT stake
                const rootStakeBefore = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, ROOT_NETUID);
                log(`Root stake before: ${rootStakeBefore}`);

                // Add stake to both dynamic subnets (owner stake to enable emissions flow)
                const subnetStakeAmount = tao(50);
                await addStake(api, owner1Coldkey, owner1HotkeyAddress, netuid1, subnetStakeAmount);
                await addStake(api, owner2Coldkey, owner2HotkeyAddress, netuid2, subnetStakeAmount);
                log(`Added ${subnetStakeAmount} owner stake to subnets ${netuid1} and ${netuid2}`);

                // Set root claim type to Swap (swap alpha to TAO and add to ROOT stake)
                await setRootClaimType(api, stakerColdkey, "Swap");
                const claimType = await getRootClaimType(api, stakerColdkeyAddress);
                log(`Root claim type: ${claimType}`);
                expect(claimType).toBe("Swap");

                // Wait for blocks
                const blocksToWait = 25;
                log(`Waiting for ${blocksToWait} blocks for emissions to accumulate...`);
                await waitForBlocks(api, blocksToWait);

                // Debug: Check moving prices
                const movingPrice1 = await getSubnetMovingPrice(api, netuid1);
                const movingPrice2 = await getSubnetMovingPrice(api, netuid2);
                const mp1Float = Number(movingPrice1) / 2 ** 32;
                const mp2Float = Number(movingPrice2) / 2 ** 32;
                log(
                    `SubnetMovingPrice (float) - netuid1: ${mp1Float}, netuid2: ${mp2Float}, sum: ${mp1Float + mp2Float}`
                );

                const pendingDivs1 = await getPendingRootAlphaDivs(api, netuid1);
                log(`PendingRootAlphaDivs netuid1: ${pendingDivs1}`);

                // Check claimable
                const claimable = await getRootClaimable(api, owner1HotkeyAddress);
                const claimableStr = [...claimable.entries()].map(([k, v]) => `[${k}: ${v.toString()}]`).join(", ");
                log(`RootClaimable entries for hotkey1: ${claimableStr || "(none)"}`);

                // Call claim_root - with Swap type, dividends are swapped to TAO and added to ROOT stake
                await claimRoot(api, stakerColdkey, [netuid1]);
                log("Called claim_root with Swap type");

                // Get ROOT stake after claim
                const rootStakeAfter = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, ROOT_NETUID);
                log(`Root stake after claim: ${rootStakeAfter}`);

                // Check RootClaimed value
                const rootClaimed = await getRootClaimed(api, netuid1, owner1HotkeyAddress, stakerColdkeyAddress);
                log(`RootClaimed value: ${rootClaimed}`);

                // With Swap type, ROOT stake should increase (not dynamic subnet stake)
                expect(rootStakeAfter, "ROOT stake should increase after claiming with Swap type").toBeGreaterThan(
                    rootStakeBefore
                );
                log(
                    `✅ Root claim with Swap successful: ROOT stake increased from ${rootStakeBefore} to ${rootStakeAfter}`
                );
            },
        });

        it({
            id: "T0403",
            title: "should handle claim_root when no dividends are available",
            test: async () => {
                // Setup accounts
                const coldkey = generateKeyringPair("sr25519");
                const coldkeyAddress = coldkey.address;

                await forceSetBalance(api, coldkeyAddress);

                // Set root claim type to Keep
                await setRootClaimType(api, coldkey, "Keep");

                // Try to claim on a non-existent subnet (should succeed but be a no-op)
                // According to Rust tests, claiming on unrelated subnets returns Ok but does nothing
                await claimRoot(api, coldkey, [1]);

                log("✅ claim_root with no dividends executed successfully (no-op).");
            },
        });
    },
});
