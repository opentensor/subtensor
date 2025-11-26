import type { Address } from 'abitype';
import type { Account } from '../../accounts/types.js';
import { type ParseAccountErrorType } from '../../accounts/utils/parseAccount.js';
import type { SignTransactionErrorType } from '../../accounts/utils/signTransaction.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import { type AccountNotFoundErrorType, type AccountTypeNotSupportedErrorType } from '../../errors/account.js';
import { type TransactionReceiptRevertedErrorType } from '../../errors/transaction.js';
import type { ErrorType } from '../../errors/utils.js';
import type { GetAccountParameter } from '../../types/account.js';
import type { Chain, DeriveChain, GetChainParameter } from '../../types/chain.js';
import type { GetTransactionRequestKzgParameter } from '../../types/kzg.js';
import type { UnionOmit } from '../../types/utils.js';
import { type RecoverAuthorizationAddressErrorType } from '../../utils/authorization/recoverAuthorizationAddress.js';
import type { RequestErrorType } from '../../utils/buildRequest.js';
import { type AssertCurrentChainErrorType } from '../../utils/chain/assertCurrentChain.js';
import { type GetTransactionErrorReturnType } from '../../utils/errors/getTransactionError.js';
import { type FormattedTransactionRequest } from '../../utils/formatters/transactionRequest.js';
import { type AssertRequestErrorType } from '../../utils/transaction/assertRequest.js';
import { type GetChainIdErrorType } from '../public/getChainId.js';
import { type WaitForTransactionReceiptErrorType } from '../public/waitForTransactionReceipt.js';
import { type PrepareTransactionRequestErrorType } from './prepareTransactionRequest.js';
import { type SendRawTransactionSyncErrorType, type SendRawTransactionSyncReturnType } from './sendRawTransactionSync.js';
export type SendTransactionSyncRequest<chain extends Chain | undefined = Chain | undefined, chainOverride extends Chain | undefined = Chain | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>> = UnionOmit<FormattedTransactionRequest<_derivedChain>, 'from'> & GetTransactionRequestKzgParameter;
export type SendTransactionSyncParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, request extends SendTransactionSyncRequest<chain, chainOverride> = SendTransactionSyncRequest<chain, chainOverride>> = request & GetAccountParameter<account, Account | Address, true, true> & GetChainParameter<chain, chainOverride> & GetTransactionRequestKzgParameter<request> & {
    /** Polling interval (ms) to poll for the transaction receipt. @default client.pollingInterval */
    pollingInterval?: number | undefined;
    /** Whether to throw an error if the transaction was detected as reverted. @default true */
    throwOnReceiptRevert?: boolean | undefined;
    /** Timeout (ms) to wait for a response. @default Math.max(chain.blockTime * 3, 5_000) */
    timeout?: number | undefined;
};
export type SendTransactionSyncReturnType<chain extends Chain | undefined = Chain | undefined> = SendRawTransactionSyncReturnType<chain>;
export type SendTransactionSyncErrorType = ParseAccountErrorType | GetTransactionErrorReturnType<AccountNotFoundErrorType | AccountTypeNotSupportedErrorType | AssertCurrentChainErrorType | AssertRequestErrorType | GetChainIdErrorType | PrepareTransactionRequestErrorType | SendRawTransactionSyncErrorType | RecoverAuthorizationAddressErrorType | SignTransactionErrorType | TransactionReceiptRevertedErrorType | RequestErrorType> | WaitForTransactionReceiptErrorType | ErrorType;
/**
 * Creates, signs, and sends a new transaction to the network synchronously.
 * Returns the transaction receipt.
 *
 * @param client - Client to use
 * @param parameters - {@link SendTransactionSyncParameters}
 * @returns The transaction receipt. {@link SendTransactionSyncReturnType}
 *
 * @example
 * import { createWalletClient, custom } from 'viem'
 * import { mainnet } from 'viem/chains'
 * import { sendTransactionSync } from 'viem/wallet'
 *
 * const client = createWalletClient({
 *   chain: mainnet,
 *   transport: custom(window.ethereum),
 * })
 * const receipt = await sendTransactionSync(client, {
 *   account: '0xA0Cf798816D4b9b9866b5330EEa46a18382f251e',
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *   value: 1000000000000000000n,
 * })
 *
 * @example
 * // Account Hoisting
 * import { createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { sendTransactionSync } from 'viem/wallet'
 *
 * const client = createWalletClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: mainnet,
 *   transport: http(),
 * })
 * const receipt = await sendTransactionSync(client, {
 *   to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *   value: 1000000000000000000n,
 * })
 */
export declare function sendTransactionSync<chain extends Chain | undefined, account extends Account | undefined, const request extends SendTransactionSyncRequest<chain, chainOverride>, chainOverride extends Chain | undefined = undefined>(client: Client<Transport, chain, account>, parameters: SendTransactionSyncParameters<chain, account, chainOverride, request>): Promise<SendTransactionSyncReturnType<chain>>;
//# sourceMappingURL=sendTransactionSync.d.ts.map