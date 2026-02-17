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

  it("should add stake to a hotkey", async () => {
    const api = await getDevnetApi();
    const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
    const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake
    const stakeAmount = tao(100);
    await addStake(api, netuid, hotkeyAddress, stakeAmount, coldkey);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    assert.ok(stakeAfter > stakeBefore, `Stake should increase: before=${stakeBefore}, after=${stakeAfter}`);

    log.info("✅ Successfully added stake.");
  });
});
