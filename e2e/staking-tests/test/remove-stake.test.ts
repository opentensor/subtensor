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
  removeStake,
  getStake,
  tao,
  log,
} from "shared";

describe("▶ remove_stake extrinsic", () => {
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

  it("should remove stake from a hotkey", async () => {
    const api = await getDevnetApi();

    // Add stake first
    await addStake(api, coldkey, hotkeyAddress, netuid, tao(200));

    // Get initial stake and balance
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    assert.ok(stakeBefore > 0n, "Should have stake before removal");

    // Remove stake (amount is in alpha units)
    const unstakeAmount = stakeBefore / 2n;
    await removeStake(api, coldkey, hotkeyAddress, netuid, unstakeAmount);

    // Verify balance increased (received TAO from unstaking)
    const balanceAfter = await getBalance(api, coldkeyAddress);
    assert.ok(balanceAfter > balanceBefore, `Balance should increase: before=${balanceBefore}, after=${balanceAfter}`);

    log.info("✅ Successfully removed stake.");
  });
});
