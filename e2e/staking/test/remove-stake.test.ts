import { describe, it, expect, beforeAll } from "vitest";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertPublicKeyToSs58,
  forceSetBalance,
  getBalance,
  addNewSubnetwork,
  startCall,
  addStake,
  removeStake,
  getStake,
  getStakeRaw,
  tao,
  log,
} from "e2e-shared";

describe("▶ remove_stake extrinsic", () => {
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

  it("should remove stake from a hotkey", async () => {
    const api = await getDevnetApi(DEFAULT_RPC_URL);

    // Add stake first
    await addStake(api, coldkey, hotkeyAddress, netuid, tao(200));

    // Get initial stake and balance (converted from U64F64 for display)
    const stakeBefore = await getStake(api, hotkeyAddress, coldkeyAddress, netuid);
    const balanceBefore = await getBalance(api, coldkeyAddress);
    expect(stakeBefore, "Should have stake before removal").toBeGreaterThan(0n);

    // Remove stake (amount is in alpha units - use raw U64F64 value)
    const stakeRaw = await getStakeRaw(api, hotkeyAddress, coldkeyAddress, netuid);
    const unstakeAmount = stakeRaw / 2n;
    await removeStake(api, coldkey, hotkeyAddress, netuid, unstakeAmount);

    // Verify balance increased (received TAO from unstaking)
    const balanceAfter = await getBalance(api, coldkeyAddress);
    expect(balanceAfter, "Balance should increase after unstaking").toBeGreaterThan(balanceBefore);

    log.info("✅ Successfully removed stake.");
  });
});
