import * as assert from "assert";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  burnedRegister,
  startCall,
  addStakeLimit,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ add_stake_limit extrinsic", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  let netuid: number;

  before(async () => {
    const api = await getDevnetApi();

    // Fund accounts
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);

    // Create subnet and register hotkey
    netuid = await addNewSubnetwork(api, hotkey, coldkey);
    await burnedRegister(api, netuid, hotkeyAddress, coldkey);
    await startCall(api, netuid, coldkey);
  });

  it("should add stake with price limit (allow partial)", async () => {
    const api = await getDevnetApi();
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake with limit price (1 TAO per Alpha) and allow partial fills
    const stakeAmount = tao(100);
    const limitPrice = tao(1); // 1 TAO per Alpha (1e9 RAO)
    await addStakeLimit(api, netuid, hotkeyAddress, stakeAmount, limitPrice, true, coldkey);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    assert.ok(stakeAfter > stakeBefore, `Stake should increase: before=${stakeBefore}, after=${stakeAfter}`);

    log.info("✅ Successfully added stake with limit (allow partial).");
  });

  it("should add stake with price limit (fill or kill)", async () => {
    const api = await getDevnetApi();
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake with high limit price (fill or kill mode)
    const stakeAmount = tao(50);
    const limitPrice = tao(10); // High limit price to ensure full fill
    await addStakeLimit(api, netuid, hotkeyAddress, stakeAmount, limitPrice, false, coldkey);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    assert.ok(stakeAfter > stakeBefore, `Stake should increase: before=${stakeBefore}, after=${stakeAfter}`);

    log.info("✅ Successfully added stake with limit (fill or kill).");
  });
});
