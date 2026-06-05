import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { subtensor } from "@polkadot-api/descriptors";
import { ethers } from "ethers";
import type { TypedApi } from "polkadot-api";
import { convertH160ToSS58, forceSetBalance, raoToEth, tao, waitForFinalizedBlocks } from "../../utils";

function createEthersWallet(provider: ethers.JsonRpcProvider): ethers.Wallet {
    const account = ethers.Wallet.createRandom();
    return new ethers.Wallet(account.privateKey, provider);
}

async function estimateTransactionCost(provider: ethers.Provider, tx: ethers.TransactionRequest): Promise<bigint> {
    const feeData = await provider.getFeeData();
    const estimatedGas = await provider.estimateGas(tx);
    const gasPrice = feeData.gasPrice ?? feeData.maxFeePerGas;
    if (gasPrice == null) {
        return estimatedGas;
    }
    return estimatedGas * gasPrice;
}

describeSuite({
    id: "evm-substrate-transfer-basic",
    title: "Basic EVM-Substrate Transfer Tests",
    foundationMethods: "zombie",
    testCases: ({ it, context }) => {
        let api: TypedApi<typeof subtensor>;
        let ethWallet: ethers.Wallet;
        let ethWallet2: ethers.Wallet;

        beforeAll(async () => {
            api = context.papi("Node").getTypedApi(subtensor);

            const provider = context.ethers("EVM").provider as ethers.JsonRpcProvider;
            ethWallet = createEthersWallet(provider);
            ethWallet2 = createEthersWallet(provider);

            await forceSetBalance(api, convertH160ToSS58(ethWallet.address));
            await forceSetBalance(api, convertH160ToSS58(ethWallet2.address));
            await waitForFinalizedBlocks(api, 1);
        }, 120000);

        it({
            id: "T01",
            title: "Can transfer token from EVM to EVM",
            test: async () => {
                const provider = ethWallet.provider;
                if (provider == null) {
                    throw new Error("ethWallet has no provider");
                }

                const senderBalanceBefore = await provider.getBalance(ethWallet.address);
                const receiverBalanceBefore = await provider.getBalance(ethWallet2.address);

                const transferAmount = raoToEth(tao(1));
                const tx: ethers.TransactionRequest = {
                    to: ethWallet2.address,
                    value: transferAmount,
                };

                const txFee = await estimateTransactionCost(provider, tx);

                const txResponse = await ethWallet.sendTransaction(tx);
                const receipt = await txResponse.wait();
                expect(receipt).toBeDefined();
                expect(receipt!.status).toEqual(1);

                const senderBalanceAfter = await provider.getBalance(ethWallet.address);
                const receiverBalanceAfter = await provider.getBalance(ethWallet2.address);

                expect(senderBalanceAfter).toEqual(senderBalanceBefore - transferAmount - txFee);
                expect(receiverBalanceAfter).toEqual(receiverBalanceBefore + transferAmount);
            },
        });
    },
});
