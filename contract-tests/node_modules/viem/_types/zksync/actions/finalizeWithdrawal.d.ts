import type { Account } from '../../accounts/types.js';
import { type SendTransactionErrorType, type SendTransactionParameters, type SendTransactionRequest, type SendTransactionReturnType } from '../../actions/wallet/sendTransaction.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import { type ChainNotFoundErrorType } from '../../errors/chain.js';
import type { Chain } from '../../types/chain.js';
import type { Hex } from '../../types/misc.js';
import { type WithdrawalLogNotFoundErrorType } from '../errors/bridge.js';
import type { ChainEIP712 } from '../types/chain.js';
export type FinalizeWithdrawalParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, chainL2 extends ChainEIP712 | undefined = ChainEIP712 | undefined, accountL2 extends Account | undefined = Account | undefined, request extends SendTransactionRequest<chain, chainOverride> = SendTransactionRequest<chain, chainOverride>> = Omit<SendTransactionParameters<chain, account, chainOverride, request>, 'value' | 'data' | 'to'> & {
    /** L2 client */
    client: Client<Transport, chainL2, accountL2>;
    /** Hash of the L2 transaction where the withdrawal was initiated. */
    hash: Hex;
    /** In case there were multiple withdrawals in one transaction, you may pass an index of the
     withdrawal you want to finalize. */
    index?: number | undefined;
};
export type FinalizeWithdrawalReturnType = SendTransactionReturnType;
export type FinalizeWithdrawalErrorType = SendTransactionErrorType | WithdrawalLogNotFoundErrorType | ChainNotFoundErrorType;
/**
 * Proves the inclusion of the `L2->L1` withdrawal message.
 *
 * @param client - Client to use
 * @param parameters - {@link FinalizeWithdrawalParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link FinalizeWithdrawalReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet, zksync } from 'viem/chains'
 * import { finalizeWithdrawal, publicActionsL2 } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * }).extend(publicActionsL2())
 *
 * const hash = await finalizeWithdrawal(client, {
 *     account: privateKeyToAccount('0x…'),
 *     client: clientL2,
 *     hash: '0x...',
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet, zksync } from 'viem/chains'
 * import { finalizeWithdrawal, publicActionsL2 } from 'viem/zksync'
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
 * const hash = await finalizeWithdrawal(client, {
 *     client: clientL2,
 *     hash: '0x…',
 * })
 */
export declare function finalizeWithdrawal<chain extends Chain | undefined, account extends Account | undefined, accountL2 extends Account | undefined, const request extends SendTransactionRequest<chain, chainOverride>, chainOverride extends Chain | undefined, chainL2 extends ChainEIP712 | undefined>(client: Client<Transport, chain, account>, parameters: FinalizeWithdrawalParameters<chain, account, chainOverride, chainL2, accountL2, request>): Promise<FinalizeWithdrawalReturnType>;
//# sourceMappingURL=finalizeWithdrawal.d.ts.map