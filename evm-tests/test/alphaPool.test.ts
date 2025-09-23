import * as assert from "assert";
import * as chai from "chai";
import { u8aToHex } from "@polkadot/util";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { generateRandomEthersWallet, getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { ALPHA_POOL_CONTRACT_ABI, ALPHA_POOL_CONTRACT_BYTECODE } from "../src/contracts/alphaPool";
import { convertH160ToPublicKey, convertH160ToSS58, convertPublicKeyToSs58, toViemAddress } from "../src/address-utils";
import { forceSetBalanceToEthAddress, disableWhiteListCheck, addNewSubnetwork, forceSetBalanceToSs58Address, startCall, burnedRegister } from "../src/subtensor";
import { ethers } from "ethers"
import { tao } from "../src/balance-math";
import { ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/contracts/staking";
// import { KeyPair } from "@polkadot-labs/hdkd-helpers";
describe("bridge token contract deployment", () => {
  // init eth part
  const wallet = generateRandomEthersWallet();
  let publicClient: PublicClient;

  // init substrate part
  let api: TypedApi<typeof devnet>
  const hotkey = getRandomSubstrateKeypair();
  const coldkey = getRandomSubstrateKeypair();

  before(async () => {
    // init variables got from await and async
    publicClient = await getPublicClient(ETH_LOCAL_URL)
    api = await getDevnetApi()

    await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
    await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
    await addNewSubnetwork(api, hotkey, coldkey)
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
    await startCall(api, netuid, coldkey)
    console.log("will test in subnet: ", netuid)

    await burnedRegister(api, netuid, convertH160ToSS58(wallet.address), coldkey)

    await forceSetBalanceToEthAddress(api, wallet.address)
    await disableWhiteListCheck(api, true)
  });

  it("Can add stake V2", async () => {
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
    // the unit in V2 is RAO, not ETH
    let stakeBalance = tao(20)
    const stakeBefore = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet.address), netuid)
    const contract = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet);
    const tx = await contract.addStake(hotkey.publicKey, stakeBalance.toString(), netuid)
    await tx.wait()

    const stakeFromContract = BigInt(
      await contract.getStake(hotkey.publicKey, convertH160ToPublicKey(wallet.address), netuid)
    );

    assert.ok(stakeFromContract > stakeBefore)
    const stakeAfter = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet.address), netuid)
    assert.ok(stakeAfter > stakeBefore)
  })


  it("Can deploy alpha pool smart contract", async () => {
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
    const contractFactory = new ethers.ContractFactory(ALPHA_POOL_CONTRACT_ABI, ALPHA_POOL_CONTRACT_BYTECODE, wallet)
    const contract = await contractFactory.deploy(hotkey.publicKey)
    await contract.waitForDeployment()
    assert.notEqual(contract.target, undefined)

    const contractAddress = contract.target.toString()
    const contractPublicKey = convertH160ToPublicKey(contractAddress)

    const code = await publicClient.getCode({ address: toViemAddress(contractAddress) })
    if (code === undefined) {
      throw new Error("code not available")
    }
    assert.ok(code.length > 100)
    assert.ok(code.includes("0x60806040523480156"))

    const contractForCall = new ethers.Contract(contractAddress, ALPHA_POOL_CONTRACT_ABI, wallet)
    const setContractColdkeyTx = await contractForCall.setContractColdkey(contractPublicKey)
    await setContractColdkeyTx.wait()

    const contractColdkey = await contractForCall.contract_coldkey()
    assert.equal(contractColdkey, u8aToHex(contractPublicKey))
    const contractHotkey = await contractForCall.contract_hotkey()
    assert.equal(contractHotkey, u8aToHex(hotkey.publicKey))

    const depositAlphaTx = await contractForCall.depositAlpha(netuid, tao(10), hotkey.publicKey)
    await depositAlphaTx.wait()

    const alphaOnChain = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet.address), netuid)
    console.log("alphaOnChain", alphaOnChain)

    const alphaOnChain2 = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(contractPublicKey), convertH160ToSS58(wallet.address), netuid)
    console.log("alphaOnChain", alphaOnChain2)

    const alphaBalance = await contractForCall.alphaBalance(convertH160ToSS58(wallet.address), netuid)
    assert.equal(alphaBalance, tao(10))
    console.log("alphaBalance", alphaBalance)
  });

});
