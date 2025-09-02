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
import { ethers } from "ethers"
import { estimateTransactionCost, getContract } from "../src/eth"

import { WITHDRAW_CONTRACT_ABI, WITHDRAW_CONTRACT_BYTECODE } from "../src/contracts/withdraw"

import { forceSetBalanceToEthAddress, forceSetBalanceToSs58Address, disableWhiteListCheck } from "../src/subtensor";

describe("Balance transfers between substrate and EVM", () => {
    const gwei = BigInt("1000000000");
    // init eth part
    const wallet = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    let publicClient: PublicClient;
    const provider = new ethers.JsonRpcProvider(ETH_LOCAL_URL);
    // init substrate part
    const signer = getRandomSubstrateSigner();
    let api: TypedApi<typeof devnet>

    before(async () => {

        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()

        await forceSetBalanceToEthAddress(api, wallet.address)
        await forceSetBalanceToEthAddress(api, wallet2.address)
        await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(signer.publicKey))
        await disableWhiteListCheck(api, true)
    });

    it("Can transfer token from EVM to EVM", async () => {
        const senderBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const receiverBalance = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        const transferBalance = raoToEth(tao(1))
        const tx = {
            to: wallet2.address,
            value: transferBalance.toString()
        }
        const txFee = await estimateTransactionCost(provider, tx)

        const txResponse = await wallet.sendTransaction(tx)
        await txResponse.wait();


        const senderBalanceAfterTransfer = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const receiverBalanceAfterTranser = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })

        assert.equal(senderBalanceAfterTransfer, senderBalance - transferBalance - txFee)
        assert.equal(receiverBalance, receiverBalanceAfterTranser - transferBalance)
    });

    it("Can transfer token from Substrate to EVM", async () => {
        const ss58Address = convertH160ToSS58(wallet.address)
        const senderBalance = (await api.query.System.Account.getValue(ss58Address)).data.free
        const receiverBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const transferBalance = tao(1)

        const tx = api.tx.Balances.transfer_keep_alive({ value: transferBalance, dest: MultiAddress.Id(ss58Address) })
        await waitForTransactionWithRetry(api, tx, signer)

        const senderBalanceAfterTransfer = (await api.query.System.Account.getValue(ss58Address)).data.free
        const receiverBalanceAfterTranser = await publicClient.getBalance({ address: toViemAddress(wallet.address) })

        assert.equal(senderBalanceAfterTransfer, senderBalance + transferBalance)
        assert.equal(receiverBalance, receiverBalanceAfterTranser - raoToEth(transferBalance))
    });

    it("Can transfer token from EVM to Substrate", async () => {
        const contract = getContract(IBALANCETRANSFER_ADDRESS, IBalanceTransferABI, wallet)
        const senderBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const receiverBalance = (await api.query.System.Account.getValue(convertPublicKeyToSs58(signer.publicKey))).data.free
        const transferBalance = raoToEth(tao(1))

        const tx = await contract.transfer(signer.publicKey, { value: transferBalance.toString() })
        await tx.wait()


        const senderBalanceAfterTransfer = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const receiverBalanceAfterTranser = (await api.query.System.Account.getValue(convertPublicKeyToSs58(signer.publicKey))).data.free

        compareEthBalanceWithTxFee(senderBalanceAfterTransfer, senderBalance - transferBalance)
        assert.equal(receiverBalance, receiverBalanceAfterTranser - tao(1))
    });

    it("Transfer from EVM to substrate using evm::withdraw", async () => {
        const ss58Address = convertPublicKeyToSs58(signer.publicKey)
        const senderBalance = (await api.query.System.Account.getValue(ss58Address)).data.free
        const ethAddresss = ss58ToH160(ss58Address);

        // transfer token to mirror eth address
        const ethTransfer = {
            to: ss58ToEthAddress(ss58Address),
            value: raoToEth(tao(2)).toString()
        }

        const txResponse = await wallet.sendTransaction(ethTransfer)
        await txResponse.wait();

        const tx = api.tx.EVM.withdraw({ address: ethAddresss, value: tao(1) })
        const txFee = (await tx.getPaymentInfo(ss58Address)).partial_fee

        await waitForTransactionWithRetry(api, tx, signer)

        const senderBalanceAfterWithdraw = (await api.query.System.Account.getValue(ss58Address)).data.free

        assert.equal(senderBalance, senderBalanceAfterWithdraw - tao(1) + txFee)
    });

    it("Transfer from EVM to substrate using evm::call", async () => {
        const ss58Address = convertPublicKeyToSs58(signer.publicKey)
        const ethAddresss = ss58ToH160(ss58Address);

        // transfer token to mirror eth address
        const ethTransfer = {
            to: ss58ToEthAddress(ss58Address),
            value: raoToEth(tao(2)).toString()
        }

        const txResponse = await wallet.sendTransaction(ethTransfer)
        await txResponse.wait();

        const source: FixedSizeBinary<20> = ethAddresss;
        const target = ethAddressToH160(wallet.address)
        const receiverBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })

        // all these parameter value are tricky, any change could make the call failed
        const tx = api.tx.EVM.call({
            source: source,
            target: target,
            // it is U256 in the extrinsic. 
            value: [raoToEth(tao(1)), tao(0), tao(0), tao(0)],
            gas_limit: BigInt(1000000),
            // it is U256 in the extrinsic. 
            max_fee_per_gas: [BigInt(10e9), BigInt(0), BigInt(0), BigInt(0)],
            max_priority_fee_per_gas: undefined,
            input: Binary.fromText(""),
            nonce: undefined,
            access_list: []
        })
        // txFee not accurate
        const txFee = (await tx.getPaymentInfo(ss58Address)).partial_fee

        await waitForTransactionWithRetry(api, tx, signer)

        const receiverBalanceAfterCall = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        assert.equal(receiverBalanceAfterCall, receiverBalance + raoToEth(tao(1)))
    });

    it("Forward value in smart contract", async () => {


        const contractFactory = new ethers.ContractFactory(WITHDRAW_CONTRACT_ABI, WITHDRAW_CONTRACT_BYTECODE, wallet)
        const contract = await contractFactory.deploy()
        await contract.waitForDeployment()

        const code = await publicClient.getCode({ address: toViemAddress(contract.target.toString()) })
        if (code === undefined) {
            throw new Error("code length is wrong for deployed contract")
        }
        assert.ok(code.length > 100)

        // transfer 2 TAO to contract
        const ethTransfer = {
            to: contract.target.toString(),
            value: raoToEth(tao(2)).toString()
        }

        const txResponse = await wallet.sendTransaction(ethTransfer)
        await txResponse.wait();

        const contractBalance = await publicClient.getBalance({ address: toViemAddress(contract.target.toString()) })
        const callerBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })

        const contractForCall = new ethers.Contract(contract.target.toString(), WITHDRAW_CONTRACT_ABI, wallet)

        const withdrawTx = await contractForCall.withdraw(
            raoToEth(tao(1)).toString()
        );

        await withdrawTx.wait();

        const contractBalanceAfterWithdraw = await publicClient.getBalance({ address: toViemAddress(contract.target.toString()) })
        const callerBalanceAfterWithdraw = await publicClient.getBalance({ address: toViemAddress(wallet.address) })

        compareEthBalanceWithTxFee(callerBalanceAfterWithdraw, callerBalance + raoToEth(tao(1)))
        assert.equal(contractBalance, contractBalanceAfterWithdraw + raoToEth(tao(1)))
    });

    it("Transfer full balance", async () => {
        const ethBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const receiverBalance = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        const tx = {
            to: wallet2.address,
            value: ethBalance.toString(),
        };
        const txPrice = await estimateTransactionCost(provider, tx);
        const finalTx = {
            to: wallet2.address,
            value: (ethBalance - txPrice).toString(),
        };
        try {
            // transfer should be failed since substrate requires existial balance to keep account
            const txResponse = await wallet.sendTransaction(finalTx)
            await txResponse.wait();
        } catch (error) {
            if (error instanceof Error) {
                assert.equal((error as any).code, "INSUFFICIENT_FUNDS")
                assert.equal(error.toString().includes("insufficient funds"), true)
            }
        }

        const receiverBalanceAfterTransfer = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        assert.equal(receiverBalance, receiverBalanceAfterTransfer)
    })

    it("Transfer more than owned balance should fail", async () => {
        const ethBalance = await publicClient.getBalance({ address: toViemAddress(wallet.address) })
        const receiverBalance = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        const tx = {
            to: wallet2.address,
            value: (ethBalance + raoToEth(tao(1))).toString(),
        };

        try {
            // transfer should be failed since substrate requires existial balance to keep account
            const txResponse = await wallet.sendTransaction(tx)
            await txResponse.wait();
        } catch (error) {
            if (error instanceof Error) {
                assert.equal((error as any).code, "INSUFFICIENT_FUNDS")
                assert.equal(error.toString().includes("insufficient funds"), true)
            }
        }

        const receiverBalanceAfterTransfer = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        assert.equal(receiverBalance, receiverBalanceAfterTransfer)
    });

    it("Transfer more than u64::max in substrate equivalent should receive error response", async () => {
        const receiverBalance = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        try {
            const tx = {
                to: wallet2.address,
                value: raoToEth(BigInt(2) ** BigInt(64)).toString(),
            };
            // transfer should be failed since substrate requires existial balance to keep account
            const txResponse = await wallet.sendTransaction(tx)
            await txResponse.wait();
        } catch (error) {
            if (error instanceof Error) {
                assert.equal((error as any).code, "INSUFFICIENT_FUNDS")
                assert.equal(error.toString().includes("insufficient funds"), true)
            }
        }

        const contract = getContract(IBALANCETRANSFER_ADDRESS, IBalanceTransferABI, wallet)
        try {
            const tx = await contract.transfer(signer.publicKey, { value: raoToEth(BigInt(2) ** BigInt(64)).toString() })
            await tx.await()
        } catch (error) {
            if (error instanceof Error) {
                console.log(error.toString())
                assert.equal(error.toString().includes("revert data"), true)
            }
        }

        try {
            const dest = convertH160ToSS58(wallet2.address)
            const tx = api.tx.Balances.transfer_keep_alive({ value: bigintToRao(BigInt(2) ** BigInt(64)), dest: MultiAddress.Id(dest) })
            await waitForTransactionCompletion(api, tx, signer)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        } catch (error) {
            if (error instanceof Error) {
                console.log(error.toString())
                assert.equal(error.toString().includes("Cannot convert"), true)
            }
        }

        try {
            const dest = ethAddressToH160(wallet2.address)
            const tx = api.tx.EVM.withdraw({ value: bigintToRao(BigInt(2) ** BigInt(64)), address: dest })
            await waitForTransactionCompletion(api, tx, signer)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        } catch (error) {
            if (error instanceof Error) {
                assert.equal(error.toString().includes("Cannot convert"), true)
            }
        }

        try {
            const source = ethAddressToH160(wallet.address)
            const target = ethAddressToH160(wallet2.address)
            const tx = api.tx.EVM.call({
                source: source,
                target: target,
                // it is U256 in the extrinsic, the value is more than u64::MAX
                value: [raoToEth(tao(1)), tao(0), tao(0), tao(1)],
                gas_limit: BigInt(1000000),
                // it is U256 in the extrinsic. 
                max_fee_per_gas: [BigInt(10e9), BigInt(0), BigInt(0), BigInt(0)],
                max_priority_fee_per_gas: undefined,
                input: Binary.fromText(""),
                nonce: undefined,
                access_list: []
            })
            await waitForTransactionCompletion(api, tx, signer)
                .then(() => { })
                .catch((error) => { console.log(`transaction error ${error}`) });
        } catch (error) {
            if (error instanceof Error) {
                console.log(error.toString())
                assert.equal((error as any).code, "INSUFFICIENT_FUNDS")
                assert.equal(error.toString().includes("insufficient funds"), true)
            }
        }

        const receiverBalanceAfterTransfer = await publicClient.getBalance({ address: toViemAddress(wallet2.address) })
        assert.equal(receiverBalance, receiverBalanceAfterTransfer)
    });

    it("Gas price should be 10 GWei", async () => {
        const feeData = await provider.getFeeData();
        assert.equal(feeData.gasPrice, BigInt(10000000000));
    });


    it("max_fee_per_gas and max_priority_fee_per_gas affect transaction fee properly", async () => {

        const testCases = [
            [10, 0, 21000 * 10 * 1e9],
            [10, 10, 21000 * 10 * 1e9],
            [11, 0, 21000 * 10 * 1e9],
            [11, 1, (21000 * 10 + 21000) * 1e9],
            [11, 2, (21000 * 10 + 21000) * 1e9],
        ];

        for (let i in testCases) {
            const tc = testCases[i];
            const actualFee = await transferAndGetFee(
                wallet, wallet2, publicClient,
                gwei * BigInt(tc[0]),
                gwei * BigInt(tc[1])
            );
            assert.equal(actualFee, BigInt(tc[2]))
        }
    });

    it("Low max_fee_per_gas gets transaction rejected", async () => {
        try {
            await transferAndGetFee(wallet, wallet2, publicClient, gwei * BigInt(9), BigInt(0))
        } catch (error) {
            if (error instanceof Error) {
                console.log(error.toString())
                assert.equal(error.toString().includes("gas price less than block base fee"), true)
            }
        }
    });

    it("max_fee_per_gas lower than max_priority_fee_per_gas gets transaction rejected", async () => {
        try {
            await transferAndGetFee(wallet, wallet2, publicClient, gwei * BigInt(10), gwei * BigInt(11))
        } catch (error) {
            if (error instanceof Error) {
                assert.equal(error.toString().includes("priorityFee cannot be more than maxFee"), true)
            }
        }
    });
});

async function transferAndGetFee(wallet: ethers.Wallet, wallet2: ethers.Wallet, client: PublicClient, max_fee_per_gas: BigInt, max_priority_fee_per_gas: BigInt) {

    const ethBalanceBefore = await client.getBalance({ address: toViemAddress(wallet.address) })
    // Send TAO
    const tx = {
        to: wallet2.address,
        value: raoToEth(tao(1)).toString(),
        // EIP-1559 transaction parameters
        maxPriorityFeePerGas: max_priority_fee_per_gas.toString(),
        maxFeePerGas: max_fee_per_gas.toString(),
        gasLimit: 21000,
    };

    // Send the transaction
    const txResponse = await wallet.sendTransaction(tx);
    await txResponse.wait()

    // Check balances
    const ethBalanceAfter = await client.getBalance({ address: toViemAddress(wallet.address) })
    const fee = ethBalanceBefore - ethBalanceAfter - raoToEth(tao(1))

    return fee;
}