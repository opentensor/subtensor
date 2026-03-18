import { describe, it, expect } from "vitest";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  burnedRegister,
  startCall,
  addStake,
  swapStakeLimit,
  getStake,
  getStakeRaw,
  tao,
  log,
} from "e2e-shared";

describe("▶ swap_stake_limit extrinsic", () => {
  it("should swap stake with price limit (allow partial)", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const hotkey1 = getRandomSubstrateKeypair();
    const hotkey2 = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const hotkey1Address = convertPublicKeyToSs58(hotkey1.publicKey);
    const hotkey2Address = convertPublicKeyToSs58(hotkey2.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

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

    log.info(`Stake on netuid1 before: ${stake1Before}, Stake on netuid2 before: ${stake2Before}`);

    // Swap stake with limit price (0.99 TAO relative price limit, allow partial fills)
    // Use raw U64F64 value for the extrinsic
    const stake1Raw = await getStakeRaw(api, hotkey1Address, coldkeyAddress, netuid1);
    const swapAmount = stake1Raw / 2n;
    const limitPrice = (tao(1) * 99n) / 100n; // 0.99 TAO
    await swapStakeLimit(api, coldkey, hotkey1Address, netuid1, netuid2, swapAmount, limitPrice, true);

    // Verify stakes changed
    const stake1After = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
    const stake2After = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);

    log.info(`Stake on netuid1 after: ${stake1After}, Stake on netuid2 after: ${stake2After}`);

    expect(stake1After, "Stake on subnet1 should decrease").toBeLessThan(stake1Before);
    expect(stake2After, "Stake on subnet2 should increase").toBeGreaterThan(stake2Before);

    log.info("✅ Successfully swapped stake with price limit (allow partial).");
  });

  it("should swap stake with price limit (fill or kill)", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const hotkey1 = getRandomSubstrateKeypair();
    const hotkey2 = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const hotkey1Address = convertPublicKeyToSs58(hotkey1.publicKey);
    const hotkey2Address = convertPublicKeyToSs58(hotkey2.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

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

    log.info(`Stake on netuid1 before: ${stake1Before}, Stake on netuid2 before: ${stake2Before}`);

    // Swap stake with limit price (fill or kill mode - allow_partial = false)
    // Use raw U64F64 value for the extrinsic
    const stake1Raw = await getStakeRaw(api, hotkey1Address, coldkeyAddress, netuid1);
    const swapAmount = stake1Raw / 2n;
    const limitPrice = tao(1) / 10n; // 0.1 TAO - permissive limit to allow slippage
    await swapStakeLimit(api, coldkey, hotkey1Address, netuid1, netuid2, swapAmount, limitPrice, false);

    // Verify stakes changed
    const stake1After = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
    const stake2After = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);

    log.info(`Stake on netuid1 after: ${stake1After}, Stake on netuid2 after: ${stake2After}`);

    expect(stake1After, "Stake on subnet1 should decrease").toBeLessThan(stake1Before);
    expect(stake2After, "Stake on subnet2 should increase").toBeGreaterThan(stake2Before);

    log.info("✅ Successfully swapped stake with price limit (fill or kill).");
  });
});
