import { describe, it, expect, beforeAll } from "vitest";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  addNewSubnetwork,
  startCall,
  addStake,
  getStake,
  tao,
  log,
} from "e2e-shared";

describe("▶ add_stake extrinsic", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const hotkeyAddress = convertPublicKeyToSs58(hotkey.publicKey);
  const coldkeyAddress = convertPublicKeyToSs58(coldkey.publicKey);
  let netuid: number;

  beforeAll(async () => {
    const api = await getDevnetApi(DEFAULT_RPC_URL);
    await forceSetBalance(api, hotkeyAddress);
    await forceSetBalance(api, coldkeyAddress);
    netuid = await addNewSubnetwork(api, hotkey, coldkey);
    await startCall(api, netuid, coldkey);
  });

  it("should add stake to a hotkey", async () => {
    const api = await getDevnetApi(DEFAULT_RPC_URL);

    // Get initial stake
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);

    // Add stake
    const stakeAmount = tao(100);
    await addStake(api, coldkey, hotkeyAddress, netuid, stakeAmount);

    // Verify stake increased
    const stakeAfter = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    expect(stakeAfter, "Stake should increase after adding stake").toBeGreaterThan(stakeBefore);

    log.info("✅ Successfully added stake.");
  });
});
