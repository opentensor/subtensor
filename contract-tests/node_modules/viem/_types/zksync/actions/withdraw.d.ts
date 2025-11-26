import type { Address } from 'abitype';
import type { Account } from '../../accounts/types.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import type { GetAccountParameter } from '../../types/account.js';
import type { GetChainParameter } from '../../types/chain.js';
import type { UnionOmit } from '../../types/utils.js';
import type { ChainEIP712 } from '../types/chain.js';
import type { ZksyncTransactionRequest } from '../types/transaction.js';
import { type SendTransactionErrorType, type SendTransactionReturnType } from './sendTransaction.js';
export type WithdrawParameters<chain extends ChainEIP712 | undefined = ChainEIP712 | undefined, account extends Account | undefined = Account | undefined, chainOverride extends ChainEIP712 | undefined = ChainEIP712 | undefined> = UnionOmit<ZksyncTransactionRequest, 'from' | 'type' | 'value' | 'data' | 'to' | 'factoryDeps' | 'maxFeePerBlobGas'> & Partial<GetAccountParameter<account>> & Partial<GetChainParameter<chain, chainOverride>> & {
    /** The address of the recipient on L1. Defaults to the sender address. */
    to?: Address;
    /** The address of the token. */
    token: Address;
    /** The amount of the token to withdraw. */
    amount: bigint;
    /** The address of the bridge contract to be used. */
    bridgeAddress?: Address;
};
export type WithdrawReturnType = SendTransactionReturnType;
export type WithdrawErrorType = SendTransactionErrorType;
/**
 * Initiates the withdrawal process which withdraws ETH or any ERC20 token
 * from the associated account on L2 network to the target account on L1 network.
 *
 * @param client - Client to use
 * @param parameters - {@link WithdrawParameters}
 * @returns hash - The [Transaction](https://viem.sh/docs/glossary/terms#transaction) hash. {@link WithdrawReturnType}
 *
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync } from 'viem/chains'
 * import { withdraw, legacyEthAddress } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await withdraw(client, {
 *     account: privateKeyToAccount('0x…'),
 *     amount: 1_000_000_000_000_000_000n,
 *     token: legacyEthAddress,
 * })
 *
 * @example Account Hoisting
 * import { createPublicClient, createWalletClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync } from 'viem/chains'
 * import { withdraw, legacyEthAddress } from 'viem/zksync'
 *
 * const client = createWalletClient({
 *   account: privateKeyToAccount('0x…'),
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await withdraw(client, {
 *     amount: 1_000_000_000_000_000_000n,
 *     token: legacyEthAddress,
 * })
 *
 * @example Paymaster
 * import { createPublicClient, http } from 'viem'
 * import { privateKeyToAccount } from 'viem/accounts'
 * import { zksync } from 'viem/chains'
 * import {
 *   withdraw,
 *   legacyEthAddress,
 *   getApprovalBasedPaymasterInput
 * } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await withdraw(client, {
 *     account: privateKeyToAccount('0x…'),
 *     amount: 1_000_000_000_000_000_000n,
 *     token: legacyEthAddress,
 *     paymaster: '0x0EEc6f45108B4b806e27B81d9002e162BD910670',
 *     paymasterInput: getApprovalBasedPaymasterInput({
 *       minAllowance: 1n,
 *       token: '0x2dc3685cA34163952CF4A5395b0039c00DFa851D',
 *       innerInput: new Uint8Array(),
 *     }),
 * })
 */
export declare function withdraw<chain extends ChainEIP712 | undefined, account extends Account | undefined, chainOverride extends ChainEIP712 | undefined = undefined>(client: Client<Transport, chain, account>, parameters: WithdrawParameters<chain, account, chainOverride>): Promise<WithdrawReturnType>;
//# sourceMappingURL=withdraw.d.ts.map