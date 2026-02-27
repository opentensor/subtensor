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
  getStake,
  tao,
  log,
} from "shared";

describe("▶ add_stake extrinsic", () => {
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

  it("should add stake to a hotkey", async () => {
    const api = await getDevnetApi();

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake
    const stakeAmount = tao(100);
    await addStake(api, coldkey, hotkeyAddress, netuid, stakeAmount);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    assert.ok(stakeAfter > stakeBefore, `Stake should increase: before=${stakeBefore}, after=${stakeAfter}`);

    log.info("✅ Successfully added stake.");
  });
});
