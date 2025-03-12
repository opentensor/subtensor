
import { ethers, Provider, TransactionRequest, Wallet } from "ethers";
export async function estimateTransactionCost(provider: Provider, tx: TransactionRequest) {
    const feeData = await provider.getFeeData();
    const estimatedGas = BigInt(await provider.estimateGas(tx));
    const gasPrice = feeData.gasPrice || feeData.maxFeePerGas;
    if (gasPrice === null)
        return estimatedGas
    else
        return estimatedGas * BigInt(gasPrice);
}

export function getContract(contractAddress: string, abi: {}[], wallet: Wallet) {
    const contract = new ethers.Contract(contractAddress, abi, wallet);
    return contract

}