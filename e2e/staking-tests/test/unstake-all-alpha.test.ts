import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  startCall,
  addStake,
  unstakeAllAlpha,
  getStake,
  tao,
  log,
} from "shared";

// Root subnet netuid is 0
const ROOT_NETUID = 0;

describe("▶ unstake_all_alpha extrinsic", () => {
  it("should unstake all alpha from multiple subnets and restake to root", async () => {
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

    // Add stake to both subnets (using same hotkey as in unstake_all test)
    await addStake(api, coldkey, hotkeyAddress, netuid1, tao(100));
    await addStake(api, coldkey, hotkeyAddress, netuid2, tao(50));

    // Verify stake was added to both subnets
    const stake1Before = await getStake(api, hotkeyAddress, coldkeyAddress, netuid1);
    const stake2Before = await getStake(api, hotkeyAddress, coldkeyAddress, netuid2);
    const rootStakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, ROOT_NETUID);

    assert.ok(stake1Before > 0n, "Should have stake in subnet 1 before unstake_all_alpha");
    assert.ok(stake2Before > 0n, "Should have stake in subnet 2 before unstake_all_alpha");
    log.info(`Stake1 before: ${stake1Before}, Stake2 before: ${stake2Before}, Root stake before: ${rootStakeBefore}`);

    // Unstake all alpha - this removes stake from dynamic subnets and restakes to root
    await unstakeAllAlpha(api, coldkey, hotkeyAddress);

    // Verify stakes are removed from both dynamic subnets
    const stake1After = await getStake(api, hotkeyAddress, coldkeyAddress, netuid1);
    const stake2After = await getStake(api, hotkeyAddress, coldkeyAddress, netuid2);
    const rootStakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, ROOT_NETUID);

    log.info(`Stake1 after: ${stake1After}, Stake2 after: ${stake2After}, Root stake after: ${rootStakeAfter}`);

    assert.strictEqual(stake1After, 0n, `Stake1 should be zero after unstake_all_alpha, got ${stake1After}`);
    assert.strictEqual(stake2After, 0n, `Stake2 should be zero after unstake_all_alpha, got ${stake2After}`);
    assert.ok(rootStakeAfter > rootStakeBefore, `Root stake should increase: before=${rootStakeBefore}, after=${rootStakeAfter}`);

    log.info("✅ Successfully unstaked all alpha from multiple subnets to root.");
  });
});
