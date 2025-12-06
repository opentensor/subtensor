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
    assert.ok(stakeFromContract > tao(20))
  })


  it("Can deploy alpha pool smart contract", async () => {
    let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1
    const stakingPrecompile = new ethers.Contract(ISTAKING_V2_ADDRESS, IStakingV2ABI, wallet);

    const stakeBeforeDeposit = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet.address), netuid)

    const contractFactory = new ethers.ContractFactory(ALPHA_POOL_CONTRACT_ABI, ALPHA_POOL_CONTRACT_BYTECODE, wallet)
    const contract = await contractFactory.deploy(hotkey.publicKey)
    await contract.waitForDeployment()
    assert.notEqual(contract.target, undefined)

    const contractAddress = contract.target.toString()
    const contractPublicKey = convertH160ToPublicKey(contractAddress)
    await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(contractPublicKey))

    const code = await publicClient.getCode({ address: toViemAddress(contractAddress) })
    if (code === undefined) {
      throw new Error("code not available")
    }
    assert.ok(code.length > 100)
    assert.ok(code.includes("0x60806040523480156"))

    console.log("deployment contractAddress: ", contractAddress)

    const contractForCall = new ethers.Contract(contractAddress, ALPHA_POOL_CONTRACT_ABI, wallet)
    const setContractColdkeyTx = await contractForCall.setContractColdkey(contractPublicKey)
    await setContractColdkeyTx.wait()

    // check contract coldkey and hotkey
    const contractColdkey = await contractForCall.contract_coldkey()
    assert.equal(contractColdkey, u8aToHex(contractPublicKey))
    const contractHotkey = await contractForCall.contract_hotkey()
    assert.equal(contractHotkey, u8aToHex(hotkey.publicKey))

    const alphaInPool = await contractForCall.getContractStake(netuid)
    assert.equal(alphaInPool, BigInt(0))

    const depositAlphaTx = await contractForCall.depositAlpha(netuid, tao(10).toString(), hotkey.publicKey)
    await depositAlphaTx.wait()

    // compare wallet stake
    const stakeAftereDeposit = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(wallet.address), netuid)
    assert.ok(stakeAftereDeposit < stakeBeforeDeposit)

    // check the contract stake
    const ContractStake = await api.query.SubtensorModule.Alpha.getValue(convertPublicKeyToSs58(hotkey.publicKey), convertH160ToSS58(contractAddress), netuid)
    assert.ok(ContractStake > 0)

    // check the wallet alpha balance in contract, the actual swapped alpha could be less than alphaAmount in deposit call
    const alphaBalanceOnContract = await contractForCall.alphaBalance(wallet.address, netuid)
    assert.ok(tao(10) - alphaBalanceOnContract < BigInt(1000))

    // check the contract stake from the staking precompile
    const stakeFromContract = BigInt(
      await stakingPrecompile.getStake(hotkey.publicKey, contractPublicKey, netuid)
    );
    assert.equal(stakeFromContract, await contractForCall.getContractStake(netuid))

  });

});
