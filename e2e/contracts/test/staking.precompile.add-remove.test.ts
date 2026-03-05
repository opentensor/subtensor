import { describe, it, expect, beforeAll } from "vitest";
import { subtensor } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { ethers } from "ethers";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertH160ToSS58,
  convertH160ToPublicKey,
  convertPublicKeyToSs58,
  forceSetBalance,
  forceSetBalanceToEthAddress,
  addNewSubnetwork,
  burnedRegister,
  startCall,
  sendProxyCall,
  tao,
  log,
} from "e2e-shared";
import { generateRandomEthersWallet, raoToEth } from "../src/ethers-utils.js";
import { ISTAKING_ADDRESS, IStakingABI, ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/staking-abi.js";

describe("Test neuron precompile add remove stake", () => {
  // ETH wallets
  const wallet1 = generateRandomEthersWallet();
  const wallet2 = generateRandomEthersWallet();

  // Substrate keypairs
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const proxy = getRandomSubstrateKeypair();

  let api: TypedApi<typeof subtensor>;
  let netuid: number;

  beforeAll(async () => {
    api = await getDevnetApi();

    await forceSetBalance(api, convertPublicKeyToSs58(hotkey.publicKey));
    await forceSetBalance(api, convertPublicKeyToSs58(coldkey.publicKey));
    await forceSetBalance(api, convertPublicKeyToSs58(proxy.publicKey));
    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);

    netuid = await addNewSubnetwork(api, hotkey, coldkey);
    await startCall(api, netuid, coldkey);

    log.info(`Test the case on subnet ${netuid}`);

    await burnedRegister(api, netuid, convertH160ToSS58(wallet1.address), coldkey);
    await burnedRegister(api, netuid, convertH160ToSS58(wallet2.address), coldkey);
  });

  it("Can add stake", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    // ETH unit for V1
    const stakeBalance = raoToEth(tao(20));
    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet1.address),
      currentNetuid
    );

    const contract = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);
    const tx = await contract.addStake(hotkey.publicKey, currentNetuid, { value: stakeBalance.toString() });
    await tx.wait();

    const stakeFromContract = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), currentNetuid)
    );

    expect(stakeFromContract, "Stake from contract should increase").toBeGreaterThan(stakeBefore);

    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet1.address),
      currentNetuid
    );
    expect(stakeAfter, "Stake should increase after adding").toBeGreaterThan(stakeBefore);

    log.info("Can add stake via V1 contract");
  });

  it("Can add stake V2", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    // RAO unit for V2
    const stakeBalance = tao(20);
    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet2.address),
      currentNetuid
    );

    const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);
    const tx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), currentNetuid);
    await tx.wait();

    const stakeFromContract = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), currentNetuid)
    );

    expect(stakeFromContract, "Stake from contract should increase").toBeGreaterThan(stakeBefore);

    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet2.address),
      currentNetuid
    );
    expect(stakeAfter, "Stake should increase after adding").toBeGreaterThan(stakeBefore);

    log.info("Can add stake via V2 contract");
  });

  it("Can not add stake if subnet doesn't exist", async () => {
    // Wrong netuid
    const wrongNetuid = 12345;
    const stakeBalance = raoToEth(tao(20));
    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet1.address),
      wrongNetuid
    );

    const contract = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);

    await expect(async () => {
      const tx = await contract.addStake(hotkey.publicKey, wrongNetuid, { value: stakeBalance.toString() });
      await tx.wait();
    }).rejects.toThrow();

    const stakeFromContract = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), wrongNetuid)
    );
    expect(stakeFromContract, "Stake should not change").toBe(stakeBefore);

    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet1.address),
      wrongNetuid
    );
    expect(stakeAfter, "Stake should not change").toBe(stakeBefore);

    log.info("Cannot add stake to non-existent subnet (V1)");
  });

  it("Can not add stake V2 if subnet doesn't exist", async () => {
    // Wrong netuid
    const wrongNetuid = 12345;
    const stakeBalance = tao(20);
    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet2.address),
      wrongNetuid
    );

    const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);

    await expect(async () => {
      const tx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), wrongNetuid);
      await tx.wait();
    }).rejects.toThrow();

    const stakeFromContract = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), wrongNetuid)
    );
    expect(stakeFromContract, "Stake should not change").toBe(stakeBefore);

    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      convertH160ToSS58(wallet2.address),
      wrongNetuid
    );
    expect(stakeAfter, "Stake should not change").toBe(stakeBefore);

    log.info("Cannot add stake to non-existent subnet (V2)");
  });

  it("Can get stake via contract read method", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;

    const contractV1 = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);
    const stakeFromContractV1 = BigInt(
      await contractV1.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), currentNetuid)
    );

    const contractV2 = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);
    // Unit from contract V2 is RAO, not ETH
    const stakeFromContractV2 = Number(
      await contractV2.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), currentNetuid)
    );

    expect(stakeFromContractV1, "V1 and V2 stakes should match").toBe(tao(stakeFromContractV2));

    const totalColdkeyStakeOnSubnet = Number(
      await contractV2.getTotalColdkeyStakeOnSubnet(convertH160ToPublicKey(wallet1.address), currentNetuid)
    );

    // Check the value is not undefined and is greater than or equal to the stake from contract V2
    expect(totalColdkeyStakeOnSubnet, "Total coldkey stake should be defined").toBeDefined();
    // Is greater than or equal to the stake from contract V2 because of emission
    expect(totalColdkeyStakeOnSubnet, "Total coldkey stake should be >= stake from V2").toBeGreaterThanOrEqual(stakeFromContractV2);

    log.info("Can get stake via contract read methods");
  });

  it("Can remove stake", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    const contract = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);

    const stakeBeforeRemove = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), currentNetuid)
    );

    const stakeBalance = raoToEth(tao(10));
    const tx = await contract.removeStake(hotkey.publicKey, stakeBalance, currentNetuid);
    await tx.wait();

    const stakeAfterRemove = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet1.address), currentNetuid)
    );
    expect(stakeAfterRemove, "Stake should decrease after removal").toBeLessThan(stakeBeforeRemove);

    log.info("Can remove stake via V1 contract");
  });

  it("Can remove stake V2", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);

    const stakeBeforeRemove = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), currentNetuid)
    );

    const stakeBalance = tao(10);
    const tx = await contract.removeStake(hotkey.publicKey, stakeBalance, currentNetuid);
    await tx.wait();

    const stakeAfterRemove = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet2.address), currentNetuid)
    );

    expect(stakeAfterRemove, "Stake should decrease after removal").toBeLessThan(stakeBeforeRemove);

    log.info("Can remove stake via V2 contract");
  });

  it("Can add/remove proxy", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    // Add/remove are done in a single test case, because we can't use the same private/public key
    // between substrate and EVM, but to test the remove part, we must predefine the proxy first.

    // First, check we don't have proxies
    const ss58Address = convertH160ToSS58(wallet1.address);
    const initProxies = await api.query.Proxy.Proxies.getValue(ss58Address);
    expect(initProxies[0].length, "Should have no proxies initially").toBe(0);

    // Initialize the contract
    const contract = new ethers.Contract(ISTAKING_ADDRESS, IStakingABI, wallet1);

    // Test "add"
    let tx = await contract.addProxy(proxy.publicKey);
    await tx.wait();

    const proxiesAfterAdd = await api.query.Proxy.Proxies.getValue(ss58Address);
    expect(proxiesAfterAdd[0][0].delegate, "Proxy should be added").toBe(convertPublicKeyToSs58(proxy.publicKey));

    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      ss58Address,
      currentNetuid
    );

    const call = api.tx.SubtensorModule.add_stake({
      hotkey: convertPublicKeyToSs58(hotkey.publicKey),
      netuid: currentNetuid,
      amount_staked: tao(1),
    });
    await sendProxyCall(api, call.decodedCall, ss58Address, proxy);

    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      ss58Address,
      currentNetuid
    );

    expect(stakeAfter, "Stake should increase via proxy call").toBeGreaterThan(stakeBefore);

    // Test "remove"
    tx = await contract.removeProxy(proxy.publicKey);
    await tx.wait();

    const proxiesAfterRemove = await api.query.Proxy.Proxies.getValue(ss58Address);
    expect(proxiesAfterRemove[0].length, "Proxy should be removed").toBe(0);

    log.info("Can add/remove proxy via V1 contract");
  });

  it("Can add/remove proxy V2", async () => {
    const currentNetuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;

    // First, check we don't have proxies
    const ss58Address = convertH160ToSS58(wallet1.address);
    const initProxies = await api.query.Proxy.Proxies.getValue(ss58Address);
    expect(initProxies[0].length, "Should have no proxies initially").toBe(0);

    // Initialize the contract
    const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

    // Test "add"
    let tx = await contract.addProxy(proxy.publicKey);
    await tx.wait();

    const proxiesAfterAdd = await api.query.Proxy.Proxies.getValue(ss58Address);
    expect(proxiesAfterAdd[0][0].delegate, "Proxy should be added").toBe(convertPublicKeyToSs58(proxy.publicKey));

    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      ss58Address,
      currentNetuid
    );

    const call = api.tx.SubtensorModule.add_stake({
      hotkey: convertPublicKeyToSs58(hotkey.publicKey),
      netuid: currentNetuid,
      amount_staked: tao(1),
    });

    await sendProxyCall(api, call.decodedCall, ss58Address, proxy);

    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(
      convertPublicKeyToSs58(hotkey.publicKey),
      ss58Address,
      currentNetuid
    );

    expect(stakeAfter, "Stake should increase via proxy call").toBeGreaterThan(stakeBefore);

    // Test "remove"
    tx = await contract.removeProxy(proxy.publicKey);
    await tx.wait();

    const proxiesAfterRemove = await api.query.Proxy.Proxies.getValue(ss58Address);
    expect(proxiesAfterRemove[0].length, "Proxy should be removed").toBe(0);

    log.info("Can add/remove proxy via V2 contract");
  });
});
