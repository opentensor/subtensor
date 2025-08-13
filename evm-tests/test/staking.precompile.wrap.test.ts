import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate";
import { devnet } from "@polkadot-api/descriptors";
import { TypedApi } from "polkadot-api";
import {
  convertH160ToSS58,
  convertPublicKeyToSs58,
  ethAddressToH160,
} from "../src/address-utils";
import { tao, raoToEth } from "../src/balance-math";
import {
  addNewSubnetwork,
  addStake,
  disableWhiteListCheck,
  forceSetBalanceToEthAddress,
  forceSetBalanceToSs58Address,
  startCall,
} from "../src/subtensor";
import { ethers } from "ethers";
import { generateRandomEthersWallet } from "../src/utils";

import { abi, bytecode } from "../src/contracts/stakeWrap";

describe("Test staking precompile add from deployed contract", () => {
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();
  const wallet1 = generateRandomEthersWallet();

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
    await addNewSubnetwork(api, hotkey, coldkey);
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    await startCall(api, netuid, coldkey);
    await disableWhiteListCheck(api, true)
    console.log("will test in subnet: ", netuid);
  });

  it("Staker add and remove stake", async () => {
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;

    const contractFactory = new ethers.ContractFactory(abi, bytecode, wallet1)
    const contract = await contractFactory.deploy()
    await contract.waitForDeployment()

    // stake will remove the balance from contract, need transfer token to deployed contract
    const ethTransfer = {
      to: contract.target.toString(),
      value: raoToEth(tao(10000)).toString()
    }

    const txResponse = await wallet1.sendTransaction(ethTransfer)
    await txResponse.wait();

    const deployedContract = new ethers.Contract(
      contract.target.toString(),
      abi,
      wallet1,
    );

    const tx = await deployedContract.stake(
      hotkey.publicKey,
      netuid,
      tao(2),
    );
    await tx.wait();

    const tx2 = await deployedContract.removeStake(
      hotkey.publicKey,
      netuid,
      tao(1),
    );
    await tx2.wait();

  });

  it("Staker add stake limit", async () => {
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1;
    let ss58Address = convertH160ToSS58(wallet1.address);

    const contractFactory = new ethers.ContractFactory(abi, bytecode, wallet1)
    const contract = await contractFactory.deploy()
    await contract.waitForDeployment()


    // stake will remove the balance from contract, need transfer token to deployed contract
    const ethTransfer = {
      to: contract.target.toString(),
      value: raoToEth(tao(10000)).toString()
    }

    const txResponse = await wallet1.sendTransaction(ethTransfer)
    await txResponse.wait();

    const balance = await api.query.System.Account.getValue(convertH160ToSS58(contract.target.toString()))
    console.log(" == balance is ", balance.data.free)

    const deployedContract = new ethers.Contract(
      contract.target.toString(),
      abi,
      wallet1,
    );

    const tx = await deployedContract.stakeLimit(
      hotkey.publicKey,
      netuid,
      tao(2000),
      tao(1000),
      true,
    );
    await tx.wait();

  });
});
