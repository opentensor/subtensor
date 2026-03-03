import { describe, it, expect, beforeAll } from "vitest";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  getBalance,
  addNewSubnetwork,
  startCall,
  addStake,
  removeStakeFullLimit,
  getStake,
  tao,
  log,
} from "e2e-shared";

describe("▶ remove_stake_full_limit extrinsic", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
  const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
  let netuid: number;

  beforeAll(async () => {
    const api = await getDevnetApi();
    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);
    netuid = await addNewSubnetwork(api, hotkey, coldkey);
    await startCall(api, netuid, coldkey);
  });

  it("should remove all stake with price limit", async () => {
    const api = await getDevnetApi();

    // Add stake first
    await addStake(api, coldkey, hotkeyAddress, netuid, tao(100));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    log.info(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
    expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

    // Remove all stake with a reasonable limit price (low limit to avoid slippage rejection)
    // Using a low limit price (0.09 TAO per alpha) allows the transaction to succeed
    const limitPrice = tao(1) / 10n; // 0.1 TAO
    await removeStakeFullLimit(api, coldkey, hotkeyAddress, netuid, limitPrice);

    // Verify stake is zero
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceAfter = await getBalance(api, coldkeyAddress);
    log.info(`Stake after: ${stakeAfter}, Balance after: ${balanceAfter}`);

    expect(stakeAfter, "Stake should be zero after full removal").toBe(0n);
    expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

    log.info("✅ Successfully removed all stake with price limit.");
  });

  it("should remove all stake without price limit", async () => {
    const api = await getDevnetApi();

    // Add stake first
    await addStake(api, coldkey, hotkeyAddress, netuid, tao(100));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    log.info(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
    expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

    // Remove all stake without limit price (undefined = no slippage protection)
    await removeStakeFullLimit(api, coldkey, hotkeyAddress, netuid, undefined);

    // Verify stake is zero
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceAfter = await getBalance(api, coldkeyAddress);
    log.info(`Stake after: ${stakeAfter}, Balance after: ${balanceAfter}`);

    expect(stakeAfter, "Stake should be zero after full removal").toBe(0n);
    expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

    log.info("✅ Successfully removed all stake without price limit.");
  });
});
