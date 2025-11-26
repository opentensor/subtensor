import type { Address } from 'abitype';
import type { Account } from '../../accounts/types.js';
import { type SendTransactionErrorType, type SendTransactionReturnType } from '../../actions/wallet/sendTransaction.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { GetAccountParameter } from '../../types/account.js';
import type { Chain, DeriveChain, GetChainParameter } from '../../types/chain.js';
import type { Hex } from '../../types/misc.js';
import type { UnionEvaluate, UnionOmit } from '../../types/utils.js';
import { type FormattedTransactionRequest } from '../../utils/index.js';
import { type BaseFeeHigherThanValueErrorType } from '../errors/bridge.js';
import type { ChainEIP712 } from '../types/chain.js';
export type RequestExecuteParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>> = UnionEvaluate<UnionOmit<FormattedTransactionRequest<_derivedChain>, 'data' | 'to' | 'from'>> & Partial<GetChainParameter<chain, chainOverride>> & Partial<GetAccountParameter<account>> & {
    /** L2 client. */
    client: Client<Transport, chainL2, accountL2>;
    /** The L2 contract to be called. */
    contractAddress: Address;
    /** The input of the L2 transaction. */
    calldata: Hex;
    /** Maximum amount of L2 gas that transaction can consume during execution on L2. */
    l2GasLimit?: bigint | undefined;
    /** The amount of base token that needs to be minted on non-ETH-based L2. */
    mintValue?: bigint | undefined;
    /** The `msg.value` of L2 transaction. */
    l2Value?: bigint | undefined;
    /** An array of L2 bytecodes that will be marked as known on L2. */
    factoryDeps?: Hex[] | undefined;
    /** (currently not used) The tip the operator will receive on top of
     the base cost of the transaction. */
    operatorTip?: bigint | undefined;
    /** The L2 gas price for each published L1 calldata byte. */
    gasPerPubdataByte?: bigint | undefined;
    /** The address on L2 that will receive the refund for the transaction.
     If the transaction fails, it will also be the address to receive `l2Value`. */
    refundRecipient?: Address | undefined;
};
export type RequestExecuteReturnType = SendTransactionReturnType;
export type RequestExecuteErrorType = SendTransactionErrorType | BaseFeeHigherThanValueErrorType;
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
export declare function requestExecute<chain extends Chain | undefined, account extends Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined>(client: Client<Transport, chain, account>, parameters: RequestExecuteParameters<chain, account, chainOverride, chainL2, accountL2>): Promise<RequestExecuteReturnType>;
//# sourceMappingURL=requestExecute.d.ts.map