import * as assert from "assert";
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
  removeStakeLimit,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ remove_stake_limit extrinsic", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
  const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
  let netuid: number;

  before(async () => {
    const api = await getDevnetApi();
    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);
    netuid = await addNewSubnetwork(api, hotkey, coldkey);
    await startCall(api, netuid, coldkey);
  });

  it("should remove stake with price limit (allow partial)", async () => {
    const api = await getDevnetApi();

    // Add stake first (100 TAO like benchmark)
    await addStake(api, coldkey, hotkeyAddress, netuid, tao(100));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    log.info(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
    assert.ok(stakeBefore > 0n, "Should have stake before removal");

    // Remove stake with limit price and allow partial fills
    const unstakeAmount = tao(30);
    const limitPrice = tao(1);
    await removeStakeLimit(api, coldkey, hotkeyAddress, netuid, unstakeAmount, limitPrice, true);

    // Verify balance increased (received TAO from unstaking)
    const balanceAfter = await getBalance(api, coldkeyAddress);
    assert.ok(balanceAfter > balanceBefore, `Balance should increase: before=${balanceBefore}, after=${balanceAfter}`);

    log.info("✅ Successfully removed stake with limit (allow partial).");
  });

  it("should remove stake with price limit (fill or kill)", async () => {
    const api = await getDevnetApi();

    // Add stake first (100 TAO like benchmark)
    await addStake(api, coldkey, hotkeyAddress, netuid, tao(100));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    log.info(`Stake before: ${stakeBefore}, Balance before: ${balanceBefore}`);
    assert.ok(stakeBefore > 0n, "Should have stake before removal");

    // Remove stake with limit price (fill or kill mode)
    const unstakeAmount = tao(30);
    const limitPrice = tao(1);
    await removeStakeLimit(api, coldkey, hotkeyAddress, netuid, unstakeAmount, limitPrice, false);

    // Verify balance increased (received TAO from unstaking)
    const balanceAfter = await getBalance(api, coldkeyAddress);
    assert.ok(balanceAfter > balanceBefore, `Balance should increase: before=${balanceBefore}, after=${balanceAfter}`);

    log.info("✅ Successfully removed stake with limit (fill or kill).");
  });
});
