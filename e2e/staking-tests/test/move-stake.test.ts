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
  moveStake,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ move_stake extrinsic", () => {
  it("should move stake to another hotkey across subnets", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const originHotkey = getRandomSubstrateKeypair();
    const destinationHotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const originHotkeyAddress = convertPublicKeyToSs58(originHotkey.publicKey);
    const destinationHotkeyAddress = convertPublicKeyToSs58(destinationHotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, originHotkeyAddress);
    await forceSetBalance(api, destinationHotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);

    // Create first subnet with origin hotkey
    const netuid1 = await addNewSubnetwork(api, originHotkey, coldkey);
    await startCall(api, netuid1, coldkey);

    // Create second subnet with destination hotkey
    const netuid2 = await addNewSubnetwork(api, destinationHotkey, coldkey);
    await startCall(api, netuid2, coldkey);

    // Add stake to origin hotkey on first subnet
    await addStake(api, coldkey, originHotkeyAddress, netuid1, tao(200));

    // Get initial stakes
    const originStakeBefore = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid1);
    const destStakeBefore = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid2);
    assert.ok(originStakeBefore > 0n, "Origin hotkey should have stake before move");

    log.info(`Origin stake (netuid1) before: ${originStakeBefore}, Destination stake (netuid2) before: ${destStakeBefore}`);

    // Move stake to destination hotkey on different subnet
    const moveAmount = originStakeBefore / 2n;
    await moveStake(
      api,
      coldkey,
      originHotkeyAddress,
      destinationHotkeyAddress,
      netuid1,
      netuid2,
      moveAmount
    );

    // Verify stakes changed
    const originStakeAfter = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid1);
    const destStakeAfter = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid2);

    log.info(`Origin stake (netuid1) after: ${originStakeAfter}, Destination stake (netuid2) after: ${destStakeAfter}`);

    assert.ok(originStakeAfter < originStakeBefore, `Origin stake should decrease: before=${originStakeBefore}, after=${originStakeAfter}`);
    assert.ok(destStakeAfter > destStakeBefore, `Destination stake should increase: before=${destStakeBefore}, after=${destStakeAfter}`);

    log.info("✅ Successfully moved stake to another hotkey across subnets.");
  });

  it("should move stake to another hotkey on the same subnet", async () => {
    const api = await getDevnetApi();

    // Setup accounts
    const originHotkey = getRandomSubstrateKeypair();
    const destinationHotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const originHotkeyAddress = convertPublicKeyToSs58(originHotkey.publicKey);
    const destinationHotkeyAddress = convertPublicKeyToSs58(destinationHotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, originHotkeyAddress);
    await forceSetBalance(api, destinationHotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);

    // Create subnet with origin hotkey
    const netuid = await addNewSubnetwork(api, originHotkey, coldkey);
    await startCall(api, netuid, coldkey);

    // Register destination hotkey on the same subnet
    await burnedRegister(api, netuid, destinationHotkeyAddress, coldkey);

    // Add stake to origin hotkey
    await addStake(api, coldkey, originHotkeyAddress, netuid, tao(200));

    // Get initial stakes
    const originStakeBefore = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid);
    const destStakeBefore = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid);
    assert.ok(originStakeBefore > 0n, "Origin hotkey should have stake before move");

    log.info(`Origin stake before: ${originStakeBefore}, Destination stake before: ${destStakeBefore}`);

    // Move stake to destination hotkey on the same subnet
    const moveAmount = originStakeBefore / 2n;
    await moveStake(
      api,
      coldkey,
      originHotkeyAddress,
      destinationHotkeyAddress,
      netuid,
      netuid,
      moveAmount
    );

    // Verify stakes changed
    const originStakeAfter = await getStake(api, originHotkeyAddress, coldkeyAddress, netuid);
    const destStakeAfter = await getStake(api, destinationHotkeyAddress, coldkeyAddress, netuid);

    log.info(`Origin stake after: ${originStakeAfter}, Destination stake after: ${destStakeAfter}`);

    assert.ok(originStakeAfter < originStakeBefore, `Origin stake should decrease: before=${originStakeBefore}, after=${originStakeAfter}`);
    assert.ok(destStakeAfter > destStakeBefore, `Destination stake should increase: before=${destStakeBefore}, after=${destStakeAfter}`);

    log.info("✅ Successfully moved stake to another hotkey on the same subnet.");
  });
});
