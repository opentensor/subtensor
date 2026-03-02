import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  startCall,
  addStake,
  transferStake,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ transfer_stake extrinsic", () => {
  it("should transfer stake to another coldkey across subnets", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const hotkey1 = getRandomSubstrateKeypair();
    const hotkey2 = getRandomSubstrateKeypair();
    const originColdkey = getRandomSubstrateKeypair();
    const destinationColdkey = getRandomSubstrateKeypair();
    const hotkey1Address = convertPublicKeyToSs58(hotkey1.publicKey);
    const hotkey2Address = convertPublicKeyToSs58(hotkey2.publicKey);
    const originColdkeyAddress = convertPublicKeyToSs58(originColdkey.publicKey);
    const destinationColdkeyAddress = convertPublicKeyToSs58(destinationColdkey.publicKey);

    await forceSetBalance(api, hotkey1Address);
    await forceSetBalance(api, hotkey2Address);
    await forceSetBalance(api, originColdkeyAddress);
    await forceSetBalance(api, destinationColdkeyAddress);

    // Create first subnet
    const netuid1 = await addNewSubnetwork(api, hotkey1, originColdkey);
    await startCall(api, netuid1, originColdkey);

    // Create second subnet
    const netuid2 = await addNewSubnetwork(api, hotkey2, originColdkey);
    await startCall(api, netuid2, originColdkey);

    // Add stake from origin coldkey on first subnet
    await addStake(api, originColdkey, hotkey1Address, netuid1, tao(200));

    // Get initial stakes
    const originStakeBefore = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
    const destStakeBefore = await getStake(api, hotkey1Address, destinationColdkeyAddress, netuid2);
    assert.ok(originStakeBefore > 0n, "Origin should have stake before transfer");

    log.info(`Origin stake (netuid1) before: ${originStakeBefore}, Destination stake (netuid2) before: ${destStakeBefore}`);

    // Transfer stake to destination coldkey on a different subnet
    const transferAmount = originStakeBefore / 2n;
    await transferStake(
      api,
      originColdkey,
      destinationColdkeyAddress,
      hotkey1Address,
      netuid1,
      netuid2,
      transferAmount
    );

    // Verify stakes changed
    const originStakeAfter = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
    const destStakeAfter = await getStake(api, hotkey1Address, destinationColdkeyAddress, netuid2);

    log.info(`Origin stake (netuid1) after: ${originStakeAfter}, Destination stake (netuid2) after: ${destStakeAfter}`);

    assert.ok(originStakeAfter < originStakeBefore, `Origin stake should decrease: before=${originStakeBefore}, after=${originStakeAfter}`);
    assert.ok(destStakeAfter > destStakeBefore, `Destination stake should increase: before=${destStakeBefore}, after=${destStakeAfter}`);

    log.info("✅ Successfully transferred stake to another coldkey across subnets.");
  });

  it("should transfer stake to another coldkey", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const hotkey = getRandomSubstrateKeypair();
    const originColdkey = getRandomSubstrateKeypair();
    const destinationColdkey = getRandomSubstrateKeypair();
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const originColdkeyAddress = convertPublicKeyToSs58(originColdkey.publicKey);
    const destinationColdkeyAddress = convertPublicKeyToSs58(destinationColdkey.publicKey);

    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, originColdkeyAddress);
    await forceSetBalance(api, destinationColdkeyAddress);

    // Create subnet
    const netuid = await addNewSubnetwork(api, hotkey, originColdkey);
    await startCall(api, netuid, originColdkey);

    // Add stake from origin coldkey
    const stakeAmount = tao(100);
    await addStake(api, originColdkey, hotkeyAddress, netuid, stakeAmount);

    // Get initial stake
    const originStakeBefore = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
    assert.ok(originStakeBefore > 0n, "Origin should have stake before transfer");

    log.info(`Origin stake before: ${originStakeBefore}`);

    // Transfer stake to destination coldkey
    // Use the known staked amount instead of queried value (Alpha storage returns inflated values)
    await transferStake(
      api,
      originColdkey,
      destinationColdkeyAddress,
      hotkeyAddress,
      netuid,
      netuid,
      stakeAmount
    );

    // Verify destination received stake
    const originStakeAfter = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
    const destStakeAfter = await getStake(api, hotkeyAddress, destinationColdkeyAddress, netuid);

    log.info(`Origin stake after: ${originStakeAfter}, Destination stake after: ${destStakeAfter}`);

    assert.ok(originStakeAfter < originStakeBefore, `Origin stake should decrease after transfer`);
    assert.ok(destStakeAfter > 0n, `Destination stake should be non-zero after transfer`);

    log.info("✅ Successfully transferred stake to another coldkey.");
  });
});
