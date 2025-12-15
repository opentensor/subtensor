import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate";
import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import {
  convertH160ToSS58,
  convertPublicKeyToSs58,
} from "../src/address-utils";
import { tao, raoToEth } from "../src/balance-math";
import {
  addNewSubnetwork,
  addStake,
  forceSetBalanceToEthAddress,
  forceSetBalanceToSs58Address,
  startCall,
} from "../src/subtensor";
import { ethers } from "ethers";
import { generateRandomEthersWallet } from "../src/utils";
import { ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/contracts/staking";
import { log } from "console";

describe("Test staking precompile add remove limit methods", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const wallet1 = generateRandomEthersWallet();
  const wallet2 = generateRandomEthersWallet();

  let api: TypedApi<typeof devnet>;

  before(async () => {
    api = await getDevnetApi();
    await forceSetBalanceToSs58Address(
      api,
      convertPublicKeyToSs58(hotkey.publicKey),
    );
    await forceSetBalanceToSs58Address(
      api,
      convertPublicKeyToSs58(coldkey.publicKey),
    );
    await forceSetBalanceToEthAddress(api, wallet1.address);
    await forceSetBalanceToEthAddress(api, wallet2.address);

    await addNewSubnetwork(api, hotkey, coldkey);
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    await startCall(api, netuid, coldkey);
    console.log("will test in subnet: ", netuid);
  });

  describe("Add limit then remove stake with limit price", () => {
    it("Staker add limit for wallet 1", async () => {
      let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      let ss58Address = convertH160ToSS58(wallet1.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      const contract = new ethers.Contract(
        ISTAKING_V2_ADDRESS,
        IStakingV2ABI,
        wallet1,
      );
      await new Promise(resolve => setTimeout(resolve, 1000));
      const tx = await contract.addStakeLimit(
        hotkey.publicKey,
        tao(2000),
        tao(1000),
        true,
        netuid,
      );
      await tx.wait();

      const alphaAfterAddStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      assert.ok(alphaAfterAddStake > alpha);
    });

    it("Staker remove stake with limit price", async () => {
      let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      let ss58Address = convertH160ToSS58(wallet1.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      const contract = new ethers.Contract(
        ISTAKING_V2_ADDRESS,
        IStakingV2ABI,
        wallet1,
      );

      const tx = await contract.removeStakeFullLimit(
        hotkey.publicKey,
        netuid,
        90_000_000,
      );
      await tx.wait();

      const alphaAfterRemoveStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      assert.ok(alphaAfterRemoveStake < alpha);
    });
  });

  describe("Add limit then remove stake full", () => {

    it("Staker add limit for wallet 2", async () => {
      let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      let ss58Address = convertH160ToSS58(wallet2.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      const contract = new ethers.Contract(
        ISTAKING_V2_ADDRESS,
        IStakingV2ABI,
        wallet2,
      );
      await new Promise(resolve => setTimeout(resolve, 1000));
      const tx = await contract.addStakeLimit(
        hotkey.publicKey,
        tao(2000),
        tao(1000),
        true,
        netuid,
      );
      await tx.wait();

      const alphaAfterAddStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      assert.ok(alphaAfterAddStake > alpha);
    });

    it("Staker remove stake with full", async () => {
      let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
      let ss58Address = convertH160ToSS58(wallet2.address);

      const alpha = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      const contract = new ethers.Contract(
        ISTAKING_V2_ADDRESS,
        IStakingV2ABI,
        wallet2,
      );
      await new Promise(resolve => setTimeout(resolve, 1000));
      const tx = await contract.removeStakeFull(
        hotkey.publicKey,
        netuid,
      );
      await tx.wait();

      const alphaAfterRemoveStake = await api.query.SubtensorModule.Alpha.getValue(
        convertPublicKeyToSs58(hotkey.publicKey),
        ss58Address,
        netuid,
      );

      assert.ok(alphaAfterRemoveStake < alpha);
    });
  });
});
