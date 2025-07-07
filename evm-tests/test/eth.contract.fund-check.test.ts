import * as assert from "assert";

import { getDevnetApi, waitForTransactionCompletion, getRandomSubstrateSigner, waitForTransactionWithRetry } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL, IBALANCETRANSFER_ADDRESS, IBalanceTransferABI } from "../src/config";
import { devnet, MultiAddress } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { TypedApi, Binary, FixedSizeBinary } from "polkadot-api";
import { generateRandomEthersWallet } from "../src/utils";
import { tao, raoToEth, bigintToRao, compareEthBalanceWithTxFee } from "../src/balance-math";
import { toViemAddress, convertPublicKeyToSs58, convertH160ToSS58, ss58ToH160, ss58ToEthAddress, ethAddressToH160 } from "../src/address-utils"
import { ethers, QuickNodeProvider } from "ethers"
import { estimateTransactionCost, getContract } from "../src/eth"

import { WITHDRAW_CONTRACT_ABI, WITHDRAW_CONTRACT_BYTECODE } from "../src/contracts/withdraw"

import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, disableWhiteListCheck } from "../src/subtensor";

describe("Balance transfers between substrate and EVM", () => {
    const gwei = BigInt("1000000000");
    // init eth part
    const wallet = generateRandomEthersWallet();
    let publicClient: PublicClient;
    const provider = new ethers.JsonRpcProvider(ETH_LOCAL_URL);
    // init substrate part
    const signer = getRandomSubstrateSigner();
    let api: TypedApi<typeof devnet>

    before(async () => {

        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet.address)
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(signer.publicKey))
        await disableWhiteListCheck(api, true)
    });

    it("Fund check for smart contract", async () => {
        const contractFactory = new ethers.ContractFactory(WITHDRAW_CONTRACT_ABI, WITHDRAW_CONTRACT_BYTECODE, wallet)
        const contract = await contractFactory.deploy()
        await contract.waitForDeployment()

        const contractAddress = contract.target.toString()
        const code = await publicClient.getCode({ address: toViemAddress(contractAddress) })
        if (code === undefined) {
            throw new Error("code length is wrong for deployed contract")
        }
        assert.ok(code.length > 100)

        const contractSs58Address = convertH160ToSS58(contractAddress)
        console.log(contract.target.toString())
        console.log(contractSs58Address)
        const substrateBalance = await api.query.System.Account.getValue(contractSs58Address)
        const existentialDeposit = await api.constants.Balances.ExistentialDeposit()
        if (existentialDeposit === undefined) {
            throw new Error("code length is wrong for deployed contract")
        }
        const evmBalance = await publicClient.getBalance({ address: toViemAddress(contractAddress) })

        assert.equal(substrateBalance.data.free, existentialDeposit)
        assert.equal(evmBalance, 0)

        // transfer 2 TAO to contract
        const ethTransfer = {
            to: contract.target.toString(),
            value: raoToEth(tao(2)).toString()
        }

        const txResponse = await wallet.sendTransaction(ethTransfer)
        await txResponse.wait();

        const substrateBalanceAfterTransfer = await api.query.System.Account.getValue(contractSs58Address)
        const evmBalanceAfterTransfer = await publicClient.getBalance({ address: toViemAddress(contractAddress) })
        assert.equal(substrateBalanceAfterTransfer.data.free, tao(2) + existentialDeposit)
        assert.equal(evmBalanceAfterTransfer, raoToEth(tao(2)))

    });
});