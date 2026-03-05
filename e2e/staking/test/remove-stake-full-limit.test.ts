import { describe, it, expect, beforeAll } from "vitest";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  getBalance,
  addNewSubnetwork,
  burnedRegister,
  startCall,
  addStake,
  removeStakeFullLimit,
  getStake,
  tao,
  log,
} from "e2e-shared";

describe("▶ remove_stake_full_limit extrinsic", () => {
  // Separate owner and staker hotkeys to avoid minimum owner stake retention
  const ownerHotkey = getRandomSubstrateKeypair();
  const stakerHotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const ownerAddress = convertPublicKeyToSs58(ownerHotkey.publicKey);
  const stakerAddress = convertPublicKeyToSs58(stakerHotkey.publicKey);
  const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
  let netuid: number;

  beforeAll(async () => {
    const api = await getDevnetApi();
    await forceSetBalance(api, ownerAddress);
    await forceSetBalance(api, stakerAddress);
    await forceSetBalance(api, coldkeyAddress);
    netuid = await addNewSubnetwork(api, ownerHotkey, coldkey);
    await startCall(api, netuid, coldkey);
    // Register staker hotkey (not the owner)
    await burnedRegister(api, netuid, stakerAddress, coldkey);
  });

  it("should remove all stake with price limit", async () => {
    const api = await getDevnetApi();

    // Add stake first
    await addStake(api, coldkey, stakerAddress, netuid, tao(100));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, stakerAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    log.info(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
    expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

    // Remove all stake with a reasonable limit price (low limit to avoid slippage rejection)
    // Using a low limit price (0.09 TAO per alpha) allows the transaction to succeed
    const limitPrice = tao(1) / 10n; // 0.1 TAO
    await removeStakeFullLimit(api, coldkey, stakerAddress, netuid, limitPrice);

    // Verify stake is zero (staker is not owner, so all stake can be removed)
    const stakeAfter = await getStake(api, stakerAddress, coldkeyAddress, netuid);
    const balanceAfter = await getBalance(api, coldkeyAddress);
    log.info(`Stake after: ${stakeAfter}, Balance after: ${balanceAfter}`);

    expect(stakeAfter, "Stake should be zero after full removal").toBe(0n);
    expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

    log.info("✅ Successfully removed all stake with price limit.");
  });

  it("should remove all stake without price limit", async () => {
    const api = await getDevnetApi();

    // Add stake first
    await addStake(api, coldkey, stakerAddress, netuid, tao(100));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, stakerAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    log.info(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
    expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

    // Remove all stake without limit price (undefined = no slippage protection)
    await removeStakeFullLimit(api, coldkey, stakerAddress, netuid, undefined);

    // Verify stake is zero (staker is not owner, so all stake can be removed)
    const stakeAfter = await getStake(api, stakerAddress, coldkeyAddress, netuid);
    const balanceAfter = await getBalance(api, coldkeyAddress);
    log.info(`Stake after: ${stakeAfter}, Balance after: ${balanceAfter}`);

    expect(stakeAfter, "Stake should be zero after full removal").toBe(0n);
    expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

    log.info("✅ Successfully removed all stake without price limit.");
  });
});
