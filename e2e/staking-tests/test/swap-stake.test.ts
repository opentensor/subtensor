import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  burnedRegister,
  startCall,
  addStake,
  swapStake,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ swap_stake extrinsic", () => {
  it("should swap full stake from one subnet to another", async () => {
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

    // Get initial stake
    const stake1Before = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
    assert.ok(stake1Before > 0n, "Should have stake on subnet1 before swap");

    log.info(`Stake on netuid1 before: ${stake1Before}`);

    // Swap full stake from subnet1 to subnet2
    await swapStake(api, coldkey, hotkey1Address, netuid1, netuid2, stake1Before);

    // Verify stakes changed
    const stake1After = await getStake(api, hotkey1Address, coldkeyAddress, netuid1);
    const stake2After = await getStake(api, hotkey1Address, coldkeyAddress, netuid2);

    log.info(`Stake on netuid1 after: ${stake1After}, Stake on netuid2 after: ${stake2After}`);

    assert.strictEqual(stake1After, 0n, `Stake on subnet1 should be zero after full swap, got ${stake1After}`);
    assert.ok(stake2After > 0n, `Stake on subnet2 should be non-zero after swap`);

    log.info("✅ Successfully swapped full stake from one subnet to another.");
  });
});
