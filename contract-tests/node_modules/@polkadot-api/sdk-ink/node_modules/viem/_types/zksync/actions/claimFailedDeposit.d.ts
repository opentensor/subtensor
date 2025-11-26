import type { Account } from '../../accounts/types.js';
import { type SendTransactionErrorType, type SendTransactionReturnType } from '../../actions/wallet/sendTransaction.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { TransactionReceiptNotFoundErrorType } from '../../errors/transaction.js';
import type { GetAccountParameter } from '../../types/account.js';
import type { Chain, DeriveChain, GetChainParameter } from '../../types/chain.js';
import type { Hash } from '../../types/misc.js';
import type { UnionEvaluate, UnionOmit } from '../../types/utils.js';
import { type FormattedTransactionRequest } from '../../utils/index.js';
import { type CannotClaimSuccessfulDepositErrorType, type L2BridgeNotFoundErrorType, type LogProofNotFoundErrorType } from '../errors/bridge.js';
import type { ChainEIP712 } from '../types/chain.js';
export type ClaimFailedDepositParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>> = UnionEvaluate<UnionOmit<FormattedTransactionRequest<_derivedChain>, 'data' | 'to' | 'from'>> & Partial<GetChainParameter<chain, chainOverride>> & Partial<GetAccountParameter<account>> & {
    /** L2 client. */
    client: Client<Transport, chainL2, accountL2>;
    /** The L2 transaction hash of the failed deposit. */
    depositHash: Hash;
};
export type ClaimFailedDepositReturnType = SendTransactionReturnType;
export type ClaimFailedDepositErrorType = SendTransactionErrorType | TransactionReceiptNotFoundErrorType | CannotClaimSuccessfulDepositErrorType | LogProofNotFoundErrorType | L2BridgeNotFoundErrorType;
/**
 * Withdraws funds from the initiated deposit, which failed when finalizing on L2.
 * If the deposit L2 transaction has failed, it sends an L1 transaction calling `claimFailedDeposit` method of the
 * L1 bridge, which results in returning L1 tokens back to the depositor.
 *
 * @param client - Client to use
 * @param parameters - {@link ClaimFailedDepositParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link ClaimFailedDepositReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { claimFailedDeposit, publicActionsL2 } from 'viem/zksync'
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
 * const account = privateKeyToAccount('0x…')
 *
 * const hash = await claimFailedDeposit(client, {
 *     client: clientL2,
 *     account,
 *     depositHash: <L2_HASH_OF_FAILED_DEPOSIT>,
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync, mainnet } from 'viem/chains'
 * import { publicActionsL2 } from 'viem/zksync'
 *
 * const walletClient = createWalletClient({
 *   chain: mainnet,
 *   transport: http(),
 *   account: privateKeyToAccount('0x…'),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await claimFailedDeposit(walletClient, {
 *     client: clientL2,
 *     depositHash: <L2_HASH_OF_FAILED_DEPOSIT>,
 * })
 */
export declare function claimFailedDeposit<chain extends Chain | undefined, account extends Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>>(client: Client<Transport, chain, account>, parameters: ClaimFailedDepositParameters<chain, account, chainOverride, chainL2, accountL2>): Promise<ClaimFailedDepositReturnType>;
//# sourceMappingURL=claimFailedDeposit.d.ts.map