import * as assert from "assert";

import { getDevnetApi, getRandomSubstrateSigner, } from "../src/substrate"
import { getPublicClient } from "../src/utils";
import { ETH_LOCAL_URL, IBALANCETRANSFER_ADDRESS, IBalanceTransferABI } from "../src/config";
import { devnet } from "@polkadot-api/descriptors"
import { PublicClient } from "viem";
import { TypedApi } from "polkadot-api";
import { generateRandomEthersWallet } from "../src/utils";
import { tao, raoToEth } from "../src/balance-math";
import { toViemAddress, } from "../src/address-utils"
import { getContract } from "../src/eth"
import { forceSetBalanceToEthAddress, } from "../src/subtensor";

describe("Transaction replace tests", () => {
    // init eth part
    const wallet = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const signer = getRandomSubstrateSigner();
    let publicClient: PublicClient;
    let api: TypedApi<typeof devnet>

    before(async () => {

        publicClient = await getPublicClient(ETH_LOCAL_URL)
        api = await getDevnetApi()
        await forceSetBalanceToEthAddress(api, wallet.address)
    });

    it("Can replace simple transfer transaction", async () => {
        const transferBalance = raoToEth(tao(1))

        const gasPrice = BigInt(10e9)
        const gasLimit = BigInt(1000000)
        const nonce = await publicClient.getTransactionCount({ address: toViemAddress(wallet.address) })

        for (let i = 1; i < 10; i++) {
            const transfer = {
                to: wallet2.address,
                value: transferBalance.toString(),
                nonce: nonce,
                gasPrice: gasPrice * BigInt(i),
                gasLimit: gasLimit * BigInt(i)
            }

            try {
                await wallet.sendTransaction(transfer)
            } catch (error) {
                // ignore error, previous transaction could be mined. the nonce is wrong.
            }
            await new Promise(resolve => setTimeout(resolve, 10))
        }

        // check the node not crashed
        await forceSetBalanceToEthAddress(api, wallet.address)
    })

    it("Can replace precompile call transaction", async () => {
        const contract = getContract(IBALANCETRANSFER_ADDRESS, IBalanceTransferABI, wallet)
        const transferBalance = raoToEth(tao(1))

        const gasPrice = BigInt(10e9)
        const gasLimit = BigInt(1000000)
        const nonce = await publicClient.getTransactionCount({ address: toViemAddress(wallet.address) })

        for (let i = 1; i < 10; i++) {
            try {
                await contract.transfer(signer.publicKey, {
                    value: transferBalance.toString(),
                    nonce: nonce,
                    gasPrice: gasPrice * BigInt(i),
                    gasLimit: gasLimit * BigInt(i)
                })
            } catch (error) {
                // ignore error, previous transaction could be mined. the nonce is wrong.
            }

            await new Promise(resolve => setTimeout(resolve, 10))
        }
        // check the node not crashed
        await forceSetBalanceToEthAddress(api, wallet.address)
    })
})