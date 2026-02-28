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

  it("should add stake with price limit (allow partial)", async () => {
    const api = await getDevnetApi();

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake with limit price and allow partial fills, limit_price is MAX TAO per Alpha willing to pay.
    const stakeAmount = tao(44);
    const limitPrice = tao(6);
    await addStakeLimit(api, coldkey, hotkeyAddress, netuid, stakeAmount, limitPrice, true);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    assert.ok(stakeAfter > stakeBefore, `Stake should increase: before=${stakeBefore}, after=${stakeAfter}`);

    log.info("✅ Successfully added stake with limit (allow partial).");
  });

  it("should add stake with price limit (fill or kill)", async () => {
    const api = await getDevnetApi();

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake with limit price (fill or kill mode), limit_price is MAX TAO per Alpha willing to pay
    const stakeAmount = tao(44);
    const limitPrice = tao(6);
    await addStakeLimit(api, coldkey, hotkeyAddress, netuid, stakeAmount, limitPrice, false);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    assert.ok(stakeAfter > stakeBefore, `Stake should increase: before=${stakeBefore}, after=${stakeAfter}`);

    log.info("✅ Successfully added stake with limit (fill or kill).");
  });
});
