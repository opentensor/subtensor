import { generatePrivateKey } from '../../accounts/generatePrivateKey.js';
import { privateKeyToAddress } from '../../accounts/utils/privateKeyToAddress.js';
import { readContract } from '../../actions/public/readContract.js';
import { sendTransaction, } from '../../actions/wallet/sendTransaction.js';
import { publicActions } from '../../clients/decorators/public.js';
import { AccountNotFoundError } from '../../errors/account.js';
import { ClientChainNotConfiguredError } from '../../errors/chain.js';
import { encodeFunctionData, isAddressEqual, parseAccount, } from '../../utils/index.js';
import { bridgehubAbi } from '../constants/abis.js';
import { ethAddressInContracts } from '../constants/address.js';
import { requiredL1ToL2GasPerPubdataLimit } from '../constants/number.js';
import { BaseFeeHigherThanValueError, } from '../errors/bridge.js';
import { estimateGasL1ToL2 } from './estimateGasL1ToL2.js';
import { getBridgehubContractAddress } from './getBridgehubContractAddress.js';
/**
 * Requests execution of a L2 transaction from L1.
 *
 * @param client - Client to use
 * @param parameters - {@link RequestExecuteParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link RequestExecuteReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { requestExecute, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *     chain: mainnet,
 *     transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await requestExecute(client, {
 *     client: clientL2,
 *     account: privateKeyToAccount('0x…'),
 *     contractAddress: '0x43020e6e11cef7dce8e37baa09d9a996ac722057'
 *     calldata: '0x',
 *     l2Value: 1_000_000_000_000_000_000n,
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { requestExecute, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createWalletClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await requestExecute(client, {
 *     client: clientL2,
 *     contractAddress: '0x43020e6e11cef7dce8e37baa09d9a996ac722057'
 *     calldata: '0x',
 *     l2Value: 1_000_000_000_000_000_000n,
 * })
 */
export async function requestExecute(client, parameters) {
    let { account: account_ = client.account, chain: chain_ = client.chain, client: l2Client, contractAddress, calldata, l2Value = 0n, mintValue = 0n, operatorTip = 0n, factoryDeps = [], gasPerPubdataByte = requiredL1ToL2GasPerPubdataLimit, refundRecipient, l2GasLimit, value, gasPrice, maxFeePerGas, maxPriorityFeePerGas, ...rest } = parameters;
    const account = account_ ? parseAccount(account_) : client.account;
    if (!account)
        throw new AccountNotFoundError({
            docsPath: '/docs/actions/wallet/sendTransaction',
        });
    if (!l2Client.chain)
        throw new ClientChainNotConfiguredError();
    const bridgehub = await getBridgehubContractAddress(l2Client);
    const baseToken = await readContract(client, {
        address: bridgehub,
        abi: bridgehubAbi,
        functionName: 'baseToken',
        args: [BigInt(l2Client.chain.id)],
    });
    const isETHBasedChain = isAddressEqual(baseToken, ethAddressInContracts);
    refundRecipient ??= account.address;
    l2GasLimit ??= await estimateGasL1ToL2(l2Client, {
        chain: l2Client.chain,
        // If the `from` address is not provided, we use a random address, because
        // due to storage slot aggregation, the gas estimation will depend on the address
        // and so estimation for the zero address may be smaller than for the sender.
        account: l2Client.account ??
            parseAccount(privateKeyToAddress(generatePrivateKey())),
        data: calldata,
        to: contractAddress,
        value: l2Value,
        gasPerPubdata: gasPerPubdataByte,
        factoryDeps,
    });
    let gasPriceForEstimation = maxFeePerGas || gasPrice;
    if (!gasPriceForEstimation) {
        const estimatedFee = await getFeePrice(client);
        gasPriceForEstimation = estimatedFee.maxFeePerGas;
        maxFeePerGas = estimatedFee.maxFeePerGas;
        maxPriorityFeePerGas ??= estimatedFee.maxPriorityFeePerGas;
    }
    const baseCost = await readContract(client, {
        address: bridgehub,
        abi: bridgehubAbi,
        functionName: 'l2TransactionBaseCost',
        args: [
            BigInt(l2Client.chain.id),
            gasPriceForEstimation,
            l2GasLimit,
            gasPerPubdataByte,
        ],
    });
    const l2Costs = baseCost + operatorTip + l2Value;
    let providedValue = isETHBasedChain ? value : mintValue;
    if (!providedValue || providedValue === 0n) {
        providedValue = l2Costs;
    }
    if (baseCost > providedValue)
        throw new BaseFeeHigherThanValueError(baseCost, providedValue);
    const data = encodeFunctionData({
        abi: bridgehubAbi,
        functionName: 'requestL2TransactionDirect',
        args: [
            {
                chainId: BigInt(l2Client.chain.id),
                mintValue: providedValue,
                l2Contract: contractAddress,
                l2Value: l2Value,
                l2Calldata: calldata,
                l2GasLimit: l2GasLimit,
                l2GasPerPubdataByteLimit: gasPerPubdataByte,
                factoryDeps: factoryDeps,
                refundRecipient: refundRecipient,
            },
        ],
    });
    return await sendTransaction(client, {
        chain: chain_,
        account: account,
        to: bridgehub,
        value: isETHBasedChain ? providedValue : value,
        data,
        gasPrice,
        maxFeePerGas,
        maxPriorityFeePerGas,
        ...rest,
    });
}
async function getFeePrice(client) {
    const client_ = client.extend(publicActions);
    const block = await client_.getBlock();
    const baseFee = typeof block.baseFeePerGas !== 'bigint'
        ? await client_.getGasPrice()
        : block.baseFeePerGas;
    const maxPriorityFeePerGas = await client_.estimateMaxPriorityFeePerGas();
    return {
        maxFeePerGas: (baseFee * 3n) / 2n + maxPriorityFeePerGas,
        maxPriorityFeePerGas: maxPriorityFeePerGas,
    };
}
//# sourceMappingURL=requestExecute.js.map