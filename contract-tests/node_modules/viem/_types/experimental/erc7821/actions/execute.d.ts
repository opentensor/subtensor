import type { Address, Narrow } from 'abitype';
import { type SendTransactionErrorType } from '../../../actions/wallet/sendTransaction.js';
import type { Client } from '../../../clients/createClient.js';
import type { Transport } from '../../../clients/transports/createTransport.js';
import type { ErrorType } from '../../../errors/utils.js';
import type { Account, GetAccountParameter } from '../../../types/account.js';
import type { Calls } from '../../../types/calls.js';
import type { Chain, DeriveChain, GetChainParameter } from '../../../types/chain.js';
import type { Hex } from '../../../types/misc.js';
import type { UnionEvaluate, UnionPick } from '../../../types/utils.js';
import type { FormattedTransactionRequest } from '../../../utils/formatters/transactionRequest.js';
import { type EncodeExecuteDataErrorType } from '../utils/encodeExecuteData.js';
import { type GetExecuteErrorReturnType } from '../utils/getExecuteError.js';
export type ExecuteParameters<calls extends readonly unknown[] = readonly unknown[], chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined, chainOverride extends Chain | undefined = Chain | undefined, _derivedChain extends Chain | undefined = DeriveChain<chain, chainOverride>> = UnionEvaluate<UnionPick<FormattedTransactionRequest<_derivedChain>, 'authorizationList' | 'gas' | 'gasPrice' | 'maxFeePerGas' | 'maxPriorityFeePerGas'>> & GetAccountParameter<account, Account | Address, true, true> & GetChainParameter<chain, chainOverride> & {
    /** Address that will execute the calls. */
    address: Address;
    /** Calls to execute. */
    calls: Calls<Narrow<calls>>;
    /** Additional data to include for execution. */
    opData?: Hex | undefined;
};
export type ExecuteReturnType = Hex;
export type ExecuteErrorType = GetExecuteErrorReturnType | EncodeExecuteDataErrorType | SendTransactionErrorType | ErrorType;
/**
 * Executes call(s) using the `execute` function on an [ERC-7821-compatible contract](https://eips.ethereum.org/EIPS/eip-7821).
 *
 * @example
 * ```ts
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { execute } from 'viem/experimental/erc7821'
 *
 * const account = privateKeyToAccount('0x...')
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const hash = await execute(client, {
 *   account,
 *   calls: [{
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: 69420n,
 *     },
 *   }],
 *   to: account.address,
 * })
 * ```
 *
 * @example
 * ```ts
 * // Account Hoisting
 * import { createClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { mainnet } from 'viem/chains'
 * import { execute } from 'viem/experimental/erc7821'
 *
 * const account = privateKeyToAccount('0x...')
 *
 * const client = createClient({
 *   account,
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const hash = await execute(client, {
 *   calls: [{
 *     {
 *       data: '0xdeadbeef',
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *     },
 *     {
 *       to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       value: 69420n,
 *     },
 *   }],
 *   to: account.address,
 * })
 * ```
 *
 * @param client - Client to use.
 * @param parameters - {@link ExecuteParameters}
 * @returns Transaction hash. {@link ExecuteReturnType}
 */
export declare function execute<const calls extends readonly unknown[], chain extends Chain | undefined, account extends Account | undefined, chainOverride extends Chain | undefined = undefined>(client: Client<Transport, chain, account>, parameters: ExecuteParameters<calls, chain, account, chainOverride>): Promise<ExecuteReturnType>;
//# sourceMappingURL=execute.d.ts.map