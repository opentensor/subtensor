import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  startCall,
  getRootClaimType,
  setRootClaimType,
  getNumRootClaims,
  sudoSetNumRootClaims,
  getRootClaimThreshold,
  sudoSetRootClaimThreshold,
  addStake,
  getStake,
  claimRoot,
  getTempo,
  sudoSetTempo,
  waitForBlocks,
  getRootClaimable,
  getRootClaimed,
  isSubtokenEnabled,
  sudoSetSubtokenEnabled,
  isNetworkAdded,
  sudoSetAdminFreezeWindow,
  sudoSetEmaPriceHalvingPeriod,
  getSubnetTAO,
  getSubnetMovingPrice,
  getPendingRootAlphaDivs,
  getTaoWeight,
  getSubnetAlphaIn,
  getTotalHotkeyAlpha,
  sudoSetSubnetMovingAlpha,
  tao,
  log,
} from "shared";

describe("▶ set_root_claim_type extrinsic", () => {
  it("should set root claim type to Keep", async () => {
    const api = await getDevnetApi();

    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // Check initial claim type (default is "Swap")
    const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type before: ${claimTypeBefore}`);

    // Set root claim type to Keep
    await setRootClaimType(api, coldkey, "Keep");

    // Verify claim type changed
    const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type after: ${claimTypeAfter}`);

    assert.strictEqual(claimTypeAfter, "Keep", `Expected claim type to be Keep, got ${claimTypeAfter}`);

    log.info("✅ Successfully set root claim type to Keep.");
  });

  it("should set root claim type to Swap", async () => {
    const api = await getDevnetApi();

    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // First set to Keep so we can verify the change to Swap
    await setRootClaimType(api, coldkey, "Keep");
    const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type before: ${claimTypeBefore}`);
    assert.strictEqual(claimTypeBefore, "Keep", "Should be Keep before changing to Swap");

    // Set root claim type to Swap
    await setRootClaimType(api, coldkey, "Swap");

    // Verify claim type changed
    const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type after: ${claimTypeAfter}`);

    assert.strictEqual(claimTypeAfter, "Swap", `Expected claim type to be Swap, got ${claimTypeAfter}`);

    log.info("✅ Successfully set root claim type to Swap.");
  });

  it("should set root claim type to KeepSubnets", async () => {
    const api = await getDevnetApi();

    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // Check initial claim type (default is "Swap")
    const claimTypeBefore = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type before: ${JSON.stringify(claimTypeBefore)}`);

    // Set root claim type to KeepSubnets with specific subnets
    const subnetsToKeep = [1, 2];
    await setRootClaimType(api, coldkey, { type: "KeepSubnets", subnets: subnetsToKeep });

    // Verify claim type changed
    const claimTypeAfter = await getRootClaimType(api, coldkeyAddress);
    log.info(`Root claim type after: ${JSON.stringify(claimTypeAfter)}`);

    assert.strictEqual(typeof claimTypeAfter, "object", "Expected claim type to be an object");
    assert.strictEqual((claimTypeAfter as { type: string }).type, "KeepSubnets", "Expected type to be KeepSubnets");
    assert.deepStrictEqual((claimTypeAfter as { subnets: number[] }).subnets, subnetsToKeep, "Expected subnets to match");

    log.info("✅ Successfully set root claim type to KeepSubnets.");
  });
});

describe("▶ sudo_set_num_root_claims extrinsic", () => {
  it("should set num root claims", async () => {
    const api = await getDevnetApi();

    // Get initial value
    const numClaimsBefore = await getNumRootClaims(api);
    log.info(`Num root claims before: ${numClaimsBefore}`);

    // Set new value (different from current)
    const newValue = numClaimsBefore + 5n;
    await sudoSetNumRootClaims(api, newValue);

    // Verify value changed
    const numClaimsAfter = await getNumRootClaims(api);
    log.info(`Num root claims after: ${numClaimsAfter}`);

    assert.strictEqual(numClaimsAfter, newValue, `Expected num root claims to be ${newValue}, got ${numClaimsAfter}`);

    log.info("✅ Successfully set num root claims.");
  });
});

describe("▶ sudo_set_root_claim_threshold extrinsic", () => {
  it("should set root claim threshold for subnet", async () => {
    const api = await getDevnetApi();

    // Create a subnet to test with
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);

    const netuid = await addNewSubnetwork(api, hotkey, coldkey);
    await startCall(api, netuid, coldkey);

    // Get initial threshold
    const thresholdBefore = await getRootClaimThreshold(api, netuid);
    log.info(`Root claim threshold before: ${thresholdBefore}`);

    // Set new threshold value (MAX_ROOT_CLAIM_THRESHOLD is 10_000_000)
    // The value is stored as I96F32 fixed-point with 32 fractional bits
    const newThreshold = 1_000_000n;
    await sudoSetRootClaimThreshold(api, netuid, newThreshold);

    // Verify threshold changed
    // I96F32 encoding: newThreshold * 2^32 = 1_000_000 * 4294967296 = 4294967296000000
    const thresholdAfter = await getRootClaimThreshold(api, netuid);
    log.info(`Root claim threshold after: ${thresholdAfter}`);

    const expectedStoredValue = newThreshold * (1n << 32n); // I96F32 encoding
    assert.strictEqual(thresholdAfter, expectedStoredValue, `Expected threshold to be ${expectedStoredValue}, got ${thresholdAfter}`);

    log.info("✅ Successfully set root claim threshold.");
  });
});

// Root subnet netuid is 0
const ROOT_NETUID = 0;

describe("▶ claim_root extrinsic", () => {
  it("should claim root dividends with Keep type (stake to dynamic subnet)", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    // - owner1Hotkey/owner1Coldkey: subnet 1 owner
    // - owner2Hotkey/owner2Coldkey: subnet 2 owner (needed for root_sell_flag)
    // - stakerColdkey: the coldkey that will stake on root and claim dividends
    const owner1Hotkey = getRandomSubstrateKeypair();
    const owner1Coldkey = getRandomSubstrateKeypair();
    const owner2Hotkey = getRandomSubstrateKeypair();
    const owner2Coldkey = getRandomSubstrateKeypair();
    const stakerColdkey = getRandomSubstrateKeypair();
    const owner1HotkeyAddress = convertPublicKeyToSs58(owner1Hotkey.publicKey);
    const owner1ColdkeyAddress = convertPublicKeyToSs58(owner1Coldkey.publicKey);
    const owner2HotkeyAddress = convertPublicKeyToSs58(owner2Hotkey.publicKey);
    const owner2ColdkeyAddress = convertPublicKeyToSs58(owner2Coldkey.publicKey);
    const stakerColdkeyAddress = convertPublicKeyToSs58(stakerColdkey.publicKey);

    // Fund all accounts
    await forceSetBalance(api, owner1HotkeyAddress);
    await forceSetBalance(api, owner1ColdkeyAddress);
    await forceSetBalance(api, owner2HotkeyAddress);
    await forceSetBalance(api, owner2ColdkeyAddress);
    await forceSetBalance(api, stakerColdkeyAddress);

    // Disable admin freeze window to allow enabling subtoken for ROOT
    await sudoSetAdminFreezeWindow(api, 0);
    log.info("Admin freeze window set to 0");

    // Enable subtoken for ROOT subnet (required for staking on root)
    const subtokenEnabledBefore = await isSubtokenEnabled(api, ROOT_NETUID);
    if (!subtokenEnabledBefore) {
      await sudoSetSubtokenEnabled(api, ROOT_NETUID, true);
      const subtokenEnabledAfter = await isSubtokenEnabled(api, ROOT_NETUID);
      log.info(`ROOT subtoken enabled: ${subtokenEnabledAfter}`);
      assert.strictEqual(subtokenEnabledAfter, true, "ROOT subtoken should be enabled");
    }

    // Create TWO dynamic subnets - needed for root_sell_flag to become true
    // root_sell_flag = sum(moving_prices) > 1.0
    // Each subnet's moving price approaches 1.0 via EMA, so 2 subnets can exceed threshold
    const netuid1 = await addNewSubnetwork(api, owner1Hotkey, owner1Coldkey);
    await startCall(api, netuid1, owner1Coldkey);
    log.info(`Created subnet 1 with netuid: ${netuid1}`);

    const netuid2 = await addNewSubnetwork(api, owner2Hotkey, owner2Coldkey);
    await startCall(api, netuid2, owner2Coldkey);
    log.info(`Created subnet 2 with netuid: ${netuid2}`);

    // Set short tempo for faster emission distribution
    await sudoSetTempo(api, netuid1, 1);
    await sudoSetTempo(api, netuid2, 1);
    log.info("Set tempo to 1 for both subnets");

    // Set EMA price halving period to 1 for fast moving price convergence
    // Formula: alpha = SubnetMovingAlpha * blocks/(blocks + halving_time)
    // With halving_time=1: after 10 blocks, alpha ≈ 0.91, moving price ≈ 0.91
    // With 2 subnets at ~0.9 each, total > 1.0 enabling root_sell_flag
    await sudoSetEmaPriceHalvingPeriod(api, netuid1, 1);
    await sudoSetEmaPriceHalvingPeriod(api, netuid2, 1);
    log.info("Set EMA halving period to 1 for fast price convergence");

    // Set SubnetMovingAlpha to 1.0 (default is 0.000003 which is way too slow)
    // I96F32 encoding: 1.0 * 2^32 = 4294967296
    const movingAlpha = BigInt(4294967296); // 1.0 in I96F32
    await sudoSetSubnetMovingAlpha(api, movingAlpha);
    log.info("Set SubnetMovingAlpha to 1.0 for fast EMA convergence");

    // Set threshold to 0 to allow claiming any amount
    await sudoSetRootClaimThreshold(api, netuid1, 0n);
    await sudoSetRootClaimThreshold(api, netuid2, 0n);

    // Add stake to ROOT subnet for the staker (makes them eligible for root dividends)
    const rootStakeAmount = tao(100);
    await addStake(api, stakerColdkey, owner1HotkeyAddress, ROOT_NETUID, rootStakeAmount);
    log.info(`Added ${rootStakeAmount} stake to root subnet for staker`);

    // Verify root stake was added
    const rootStake = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, ROOT_NETUID);
    log.info(`Root stake: ${rootStake}`);
    assert.ok(rootStake > 0n, "Should have stake on root subnet");

    // Add stake to both dynamic subnets (owner stake to enable emissions flow)
    const subnetStakeAmount = tao(50);
    await addStake(api, owner1Coldkey, owner1HotkeyAddress, netuid1, subnetStakeAmount);
    await addStake(api, owner2Coldkey, owner2HotkeyAddress, netuid2, subnetStakeAmount);
    log.info(`Added ${subnetStakeAmount} owner stake to subnets ${netuid1} and ${netuid2}`);

    // Get initial stake on subnet 1 for the staker (should be 0)
    const stakerSubnetStakeBefore = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, netuid1);
    log.info(`Staker subnet stake before claim: ${stakerSubnetStakeBefore}`);

    // Set root claim type to Keep (keep alpha on subnet instead of swapping to TAO)
    await setRootClaimType(api, stakerColdkey, "Keep");
    const claimType = await getRootClaimType(api, stakerColdkeyAddress);
    log.info(`Root claim type: ${claimType}`);
    assert.strictEqual(claimType, "Keep", "Should have Keep claim type");

    // Wait for blocks to:
    // 1. Allow moving prices to converge (need sum > 1.0 for root_sell_flag)
    // 2. Accumulate PendingRootAlphaDivs
    // 3. Distribute emissions at tempo boundary
    const blocksToWait = 25;
    log.info(`Waiting for ${blocksToWait} blocks for moving prices to converge and emissions to accumulate...`);
    await waitForBlocks(api, blocksToWait);

    // Debug: Check key storage values
    const subnetTaoRoot = await getSubnetTAO(api, ROOT_NETUID);
    const subnetTao1 = await getSubnetTAO(api, netuid1);
    const subnetTao2 = await getSubnetTAO(api, netuid2);
    log.info(`SubnetTAO - ROOT: ${subnetTaoRoot}, netuid1: ${subnetTao1}, netuid2: ${subnetTao2}`);

    const movingPrice1 = await getSubnetMovingPrice(api, netuid1);
    const movingPrice2 = await getSubnetMovingPrice(api, netuid2);
    log.info(`SubnetMovingPrice - netuid1: ${movingPrice1}, netuid2: ${movingPrice2}`);
    // Note: Moving price is I96F32, so divide by 2^32 to get actual value
    const mp1Float = Number(movingPrice1) / 2**32;
    const mp2Float = Number(movingPrice2) / 2**32;
    log.info(`SubnetMovingPrice (float) - netuid1: ${mp1Float}, netuid2: ${mp2Float}, sum: ${mp1Float + mp2Float}`);

    const pendingDivs1 = await getPendingRootAlphaDivs(api, netuid1);
    const pendingDivs2 = await getPendingRootAlphaDivs(api, netuid2);
    log.info(`PendingRootAlphaDivs - netuid1: ${pendingDivs1}, netuid2: ${pendingDivs2}`);

    const taoWeight = await getTaoWeight(api);
    log.info(`TaoWeight: ${taoWeight}`);

    const alphaIn1 = await getSubnetAlphaIn(api, netuid1);
    const alphaIn2 = await getSubnetAlphaIn(api, netuid2);
    log.info(`SubnetAlphaIn - netuid1: ${alphaIn1}, netuid2: ${alphaIn2}`);

    const totalHotkeyAlpha1 = await getTotalHotkeyAlpha(api, owner1HotkeyAddress, netuid1);
    log.info(`TotalHotkeyAlpha for hotkey1 on netuid1: ${totalHotkeyAlpha1}`);

    // Check if there are any claimable dividends
    const claimable = await getRootClaimable(api, owner1HotkeyAddress);
    const claimableStr = [...claimable.entries()].map(([k, v]) => `[${k}: ${v.toString()}]`).join(", ");
    log.info(`RootClaimable entries for hotkey1: ${claimableStr || "(none)"}`);

    // Call claim_root to claim dividends for subnet 1
    await claimRoot(api, stakerColdkey, [netuid1]);
    log.info("Called claim_root");

    // Get stake on subnet 1 after claim
    const stakerSubnetStakeAfter = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, netuid1);
    log.info(`Staker subnet stake after claim: ${stakerSubnetStakeAfter}`);

    // Check RootClaimed value
    const rootClaimed = await getRootClaimed(api, netuid1, owner1HotkeyAddress, stakerColdkeyAddress);
    log.info(`RootClaimed value: ${rootClaimed}`);

    // Verify dividends were claimed
    assert.ok(
      stakerSubnetStakeAfter > stakerSubnetStakeBefore,
      `Stake should increase after claiming root dividends: before=${stakerSubnetStakeBefore}, after=${stakerSubnetStakeAfter}`
    );
    log.info(`✅ Root claim successful: stake increased from ${stakerSubnetStakeBefore} to ${stakerSubnetStakeAfter}`);
  });

  it("should claim root dividends with Swap type (swap to TAO on ROOT)", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    // - owner1Hotkey/owner1Coldkey: subnet 1 owner
    // - owner2Hotkey/owner2Coldkey: subnet 2 owner (needed for root_sell_flag)
    // - stakerColdkey: the coldkey that will stake on root and claim dividends
    const owner1Hotkey = getRandomSubstrateKeypair();
    const owner1Coldkey = getRandomSubstrateKeypair();
    const owner2Hotkey = getRandomSubstrateKeypair();
    const owner2Coldkey = getRandomSubstrateKeypair();
    const stakerColdkey = getRandomSubstrateKeypair();
    const owner1HotkeyAddress = convertPublicKeyToSs58(owner1Hotkey.publicKey);
    const owner1ColdkeyAddress = convertPublicKeyToSs58(owner1Coldkey.publicKey);
    const owner2HotkeyAddress = convertPublicKeyToSs58(owner2Hotkey.publicKey);
    const owner2ColdkeyAddress = convertPublicKeyToSs58(owner2Coldkey.publicKey);
    const stakerColdkeyAddress = convertPublicKeyToSs58(stakerColdkey.publicKey);

    // Fund all accounts
    await forceSetBalance(api, owner1HotkeyAddress);
    await forceSetBalance(api, owner1ColdkeyAddress);
    await forceSetBalance(api, owner2HotkeyAddress);
    await forceSetBalance(api, owner2ColdkeyAddress);
    await forceSetBalance(api, stakerColdkeyAddress);

    // Disable admin freeze window to allow enabling subtoken for ROOT
    await sudoSetAdminFreezeWindow(api, 0);
    log.info("Admin freeze window set to 0");

    // Create TWO dynamic subnets
    const netuid1 = await addNewSubnetwork(api, owner1Hotkey, owner1Coldkey);
    await startCall(api, netuid1, owner1Coldkey);
    log.info(`Created subnet 1 with netuid: ${netuid1}`);

    const netuid2 = await addNewSubnetwork(api, owner2Hotkey, owner2Coldkey);
    await startCall(api, netuid2, owner2Coldkey);
    log.info(`Created subnet 2 with netuid: ${netuid2}`);

    // Set short tempo for faster emission distribution
    await sudoSetTempo(api, netuid1, 1);
    await sudoSetTempo(api, netuid2, 1);
    log.info("Set tempo to 1 for both subnets");

    // Set EMA price halving period to 1 for fast moving price convergence
    await sudoSetEmaPriceHalvingPeriod(api, netuid1, 1);
    await sudoSetEmaPriceHalvingPeriod(api, netuid2, 1);
    log.info("Set EMA halving period to 1 for fast price convergence");

    // Set SubnetMovingAlpha to 1.0 (default is 0.000003 which is way too slow)
    // I96F32 encoding: 1.0 * 2^32 = 4294967296
    const movingAlpha = BigInt(4294967296); // 1.0 in I96F32
    await sudoSetSubnetMovingAlpha(api, movingAlpha);
    log.info("Set SubnetMovingAlpha to 1.0 for fast EMA convergence");

    // Set threshold to 0 to allow claiming any amount
    await sudoSetRootClaimThreshold(api, netuid1, 0n);
    await sudoSetRootClaimThreshold(api, netuid2, 0n);

    // Add stake to ROOT subnet for the staker
    const rootStakeAmount = tao(100);
    await addStake(api, stakerColdkey, owner1HotkeyAddress, ROOT_NETUID, rootStakeAmount);
    log.info(`Added ${rootStakeAmount} stake to root subnet for staker`);

    // Get initial ROOT stake
    const rootStakeBefore = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, ROOT_NETUID);
    log.info(`Root stake before: ${rootStakeBefore}`);

    // Add stake to both dynamic subnets (owner stake to enable emissions flow)
    const subnetStakeAmount = tao(50);
    await addStake(api, owner1Coldkey, owner1HotkeyAddress, netuid1, subnetStakeAmount);
    await addStake(api, owner2Coldkey, owner2HotkeyAddress, netuid2, subnetStakeAmount);
    log.info(`Added ${subnetStakeAmount} owner stake to subnets ${netuid1} and ${netuid2}`);

    // Set root claim type to Swap (swap alpha to TAO and add to ROOT stake)
    await setRootClaimType(api, stakerColdkey, "Swap");
    const claimType = await getRootClaimType(api, stakerColdkeyAddress);
    log.info(`Root claim type: ${claimType}`);
    assert.strictEqual(claimType, "Swap", "Should have Swap claim type");

    // Wait for blocks
    const blocksToWait = 25;
    log.info(`Waiting for ${blocksToWait} blocks for emissions to accumulate...`);
    await waitForBlocks(api, blocksToWait);

    // Debug: Check moving prices
    const movingPrice1 = await getSubnetMovingPrice(api, netuid1);
    const movingPrice2 = await getSubnetMovingPrice(api, netuid2);
    const mp1Float = Number(movingPrice1) / 2**32;
    const mp2Float = Number(movingPrice2) / 2**32;
    log.info(`SubnetMovingPrice (float) - netuid1: ${mp1Float}, netuid2: ${mp2Float}, sum: ${mp1Float + mp2Float}`);

    const pendingDivs1 = await getPendingRootAlphaDivs(api, netuid1);
    log.info(`PendingRootAlphaDivs netuid1: ${pendingDivs1}`);

    // Check claimable
    const claimable = await getRootClaimable(api, owner1HotkeyAddress);
    const claimableStr = [...claimable.entries()].map(([k, v]) => `[${k}: ${v.toString()}]`).join(", ");
    log.info(`RootClaimable entries for hotkey1: ${claimableStr || "(none)"}`);

    // Call claim_root - with Swap type, dividends are swapped to TAO and added to ROOT stake
    await claimRoot(api, stakerColdkey, [netuid1]);
    log.info("Called claim_root with Swap type");

    // Get ROOT stake after claim
    const rootStakeAfter = await getStake(api, owner1HotkeyAddress, stakerColdkeyAddress, ROOT_NETUID);
    log.info(`Root stake after claim: ${rootStakeAfter}`);

    // Check RootClaimed value
    const rootClaimed = await getRootClaimed(api, netuid1, owner1HotkeyAddress, stakerColdkeyAddress);
    log.info(`RootClaimed value: ${rootClaimed}`);

    // With Swap type, ROOT stake should increase (not dynamic subnet stake)
    assert.ok(
      rootStakeAfter > rootStakeBefore,
      `ROOT stake should increase after claiming with Swap type: before=${rootStakeBefore}, after=${rootStakeAfter}`
    );
    log.info(`✅ Root claim with Swap successful: ROOT stake increased from ${rootStakeBefore} to ${rootStakeAfter}`);
  });

  it("should handle claim_root when no dividends are available", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const coldkey = getRandomSubstrateKeypair();
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, coldkeyAddress);

    // Set root claim type to Keep
    await setRootClaimType(api, coldkey, "Keep");

    // Try to claim on a non-existent subnet (should succeed but be a no-op)
    // According to Rust tests, claiming on unrelated subnets returns Ok but does nothing
    await claimRoot(api, coldkey, [1]);

    log.info("✅ claim_root with no dividends executed successfully (no-op).");
  });
});
