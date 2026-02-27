import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  getBalance,
  addNewSubnetwork,
  startCall,
  addStake,
  unstakeAll,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ unstake_all extrinsic", () => {
  it("should unstake all from a hotkey across all subnets", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
    const hotkey2 = getRandomSubstrateKeypair();
    const hotkeyAddress2 = convertPublicKeyToSs58(hotkey2.publicKey);

    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);
    await forceSetBalance(api, hotkeyAddress2);

    // Create first subnet
    const netuid1 = await addNewSubnetwork(api, hotkey, coldkey);
    await startCall(api, netuid1, coldkey);

    // Create second subnet
    const netuid2 = await addNewSubnetwork(api, hotkey2, coldkey);
    await startCall(api, netuid2, coldkey);

    // Add stake to both subnets
    await addStake(api, coldkey, hotkeyAddress, netuid1, tao(100));
    await addStake(api, coldkey, hotkeyAddress, netuid2, tao(50));

    // Verify stake was added to both subnets
    const stake1Before = await getStake(api, hotkeyAddress, coldkeyAddress, netuid1);
    const stake2Before = await getStake(api, hotkeyAddress, coldkeyAddress, netuid2);
    const balanceBefore = await getBalance(api, coldkeyAddress);

    assert.ok(stake1Before > 0n, "Should have stake in subnet 1 before unstake_all");
    assert.ok(stake2Before > 0n, "Should have stake in subnet 2 before unstake_all");
    log.info(`Stake1 before: ${stake1Before}, Stake2 before: ${stake2Before}, Balance before: ${balanceBefore}`);

    // Unstake all
    await unstakeAll(api, coldkey, hotkeyAddress);

    // Verify stakes are removed from both subnets and balance increased
    const stake1After = await getStake(api, hotkeyAddress, coldkeyAddress, netuid1);
    const stake2After = await getStake(api, hotkeyAddress, coldkeyAddress, netuid2);
    const balanceAfter = await getBalance(api, coldkeyAddress);

    log.info(`Stake1 after: ${stake1After}, Stake2 after: ${stake2After}, Balance after: ${balanceAfter}`);

    assert.strictEqual(stake1After, 0n, `Stake1 should be zero after unstake_all, got ${stake1After}`);
    assert.strictEqual(stake2After, 0n, `Stake2 should be zero after unstake_all, got ${stake2After}`);
    assert.ok(balanceAfter > balanceBefore, `Balance should increase: before=${balanceBefore}, after=${balanceAfter}`);

    log.info("✅ Successfully unstaked all from multiple subnets.");
  });
});
