import * as assert from "assert";
import { subtensor } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import { ethers } from "ethers";
import {
  getDevnetApi,
  getRandomSubstrateKeypair,
  convertH160ToSS58,
  convertPublicKeyToSs58,
  forceSetBalance,
  forceSetBalanceToEthAddress,
  addNewSubnetwork,
  startCall,
  tao,
  log,
} from "e2e-shared";
import { generateRandomEthersWallet } from "../src/ethers-utils.js";
import { ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/staking-abi.js";

describe("Test staking precompile add remove limit methods", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const wallet1 = generateRandomEthersWallet();
  const wallet2 = generateRandomEthersWallet();

  let api: TypedApi<typeof subtensor>;

  before(async () => {
    api = await getDevnetApi();
    await forceSetBalance(api, convertPublicKeyToSs58(hotkey.publicKey));
    await forceSetBalance(api, convertPublicKeyToSs58(coldkey.publicKey));
    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);

    await addNewSubnetwork(api, hotkey, coldkey);
    const netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    await startCall(api, netuid, coldkey);
    log.info(`Will test in subnet: ${netuid}`);
  });

  describe("Add limit then remove stake with limit price", () => {
    it("Staker add limit for wallet 1", async () => {
      const netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      const ss58Address = convertH160ToSS58(wallet1.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

      const tx = await contract.addStakeLimit(hotkey.publicKey, tao(2000), tao(1000), true, netuid);
      await tx.wait();

      const alphaAfterAddStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      assert.ok(alphaAfterAddStake > alpha);
      log.info("✅ Wallet 1 added stake with limit");
    });

    it("Staker remove stake with limit price", async () => {
      const netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      const ss58Address = convertH160ToSS58(wallet1.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet1);

      const tx = await contract.removeStakeFullLimit(hotkey.publicKey, netuid, 90_000_000);
      await tx.wait();

      const alphaAfterRemoveStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      assert.ok(alphaAfterRemoveStake < alpha);
      log.info("✅ Wallet 1 removed stake with limit price");
    });
  });

  describe("Add limit then remove stake full", () => {
    it("Staker add limit for wallet 2", async () => {
      const netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      const ss58Address = convertH160ToSS58(wallet2.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);

      const tx = await contract.addStakeLimit(hotkey.publicKey, tao(2000), tao(1000), true, netuid);
      await tx.wait();

      const alphaAfterAddStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      assert.ok(alphaAfterAddStake > alpha);
      log.info("✅ Wallet 2 added stake with limit");
    });

    it("Staker remove stake with full", async () => {
      const netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      const ss58Address = convertH160ToSS58(wallet2.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet2);

      const tx = await contract.removeStakeFull(hotkey.publicKey, netuid);
      await tx.wait();

      const alphaAfterRemoveStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid
      );

      assert.ok(alphaAfterRemoveStake < alpha);
      log.info("✅ Wallet 2 removed stake full");
    });
  });
});
