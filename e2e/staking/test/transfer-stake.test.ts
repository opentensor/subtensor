import { describe, it, expect } from "vitest";
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
  getStakeRaw,
  tao,
  log,
} from "e2e-shared";
import { DEFAULT_RPC_URL } from "../setup.js";
describe("▶ transfer_stake extrinsic", () => {
  it("should transfer stake to another coldkey across subnets", async () => {
    const api = await getDevnetApi(DEFAULT_RPC_URL);

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

    // Get initial stakes (converted from U64F64 for display)
    const originStakeBefore = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
    const destStakeBefore = await getStake(api, hotkey1Address, destinationColdkeyAddress, netuid2);
    expect(originStakeBefore, "Origin should have stake before transfer").toBeGreaterThan(0n);

    log.info(
      `Origin stake (netuid1) before: ${originStakeBefore}, Destination stake (netuid2) before: ${destStakeBefore}`,
    );

    // Transfer stake to destination coldkey on a different subnet
    // Use raw U64F64 value for the extrinsic
    const originStakeRaw = await getStakeRaw(api, hotkey1Address, originColdkeyAddress, netuid1);
    const transferAmount = originStakeRaw / 2n;
    await transferStake(
      api,
      originColdkey,
      destinationColdkeyAddress,
      hotkey1Address,
      netuid1,
      netuid2,
      transferAmount,
    );

    // Verify stakes changed
    const originStakeAfter = await getStake(api, hotkey1Address, originColdkeyAddress, netuid1);
    const destStakeAfter = await getStake(api, hotkey1Address, destinationColdkeyAddress, netuid2);

    log.info(`Origin stake (netuid1) after: ${originStakeAfter}, Destination stake (netuid2) after: ${destStakeAfter}`);

    expect(originStakeAfter, "Origin stake should decrease").toBeLessThan(originStakeBefore);
    expect(destStakeAfter, "Destination stake should increase").toBeGreaterThan(destStakeBefore);

    log.info("✅ Successfully transferred stake to another coldkey across subnets.");
  });

  it("should transfer stake to another coldkey", async () => {
    const api = await getDevnetApi(DEFAULT_RPC_URL);

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

    // Get initial stake (converted from U64F64 for display)
    const originStakeBefore = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
    expect(originStakeBefore, "Origin should have stake before transfer").toBeGreaterThan(0n);

    log.info(`Origin stake before: ${originStakeBefore}`);

    // Transfer stake to destination coldkey
    // Use raw U64F64 value for the extrinsic, transfer half to avoid AmountTooLow error
    const originStakeRaw = await getStakeRaw(api, hotkeyAddress, originColdkeyAddress, netuid);
    const transferAmount = originStakeRaw / 2n;
    await transferStake(api, originColdkey, destinationColdkeyAddress, hotkeyAddress, netuid, netuid, transferAmount);

    // Verify destination received stake
    const originStakeAfter = await getStake(api, hotkeyAddress, originColdkeyAddress, netuid);
    const destStakeAfter = await getStake(api, hotkeyAddress, destinationColdkeyAddress, netuid);

    log.info(`Origin stake after: ${originStakeAfter}, Destination stake after: ${destStakeAfter}`);

    expect(originStakeAfter, "Origin stake should decrease after transfer").toBeLessThan(originStakeBefore);
    expect(destStakeAfter, "Destination stake should be non-zero after transfer").toBeGreaterThan(0n);

    log.info("✅ Successfully transferred stake to another coldkey.");
  });
});
