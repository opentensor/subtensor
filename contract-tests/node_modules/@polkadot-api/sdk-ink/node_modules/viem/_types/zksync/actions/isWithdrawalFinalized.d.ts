import type { Account } from '../../accounts/types.js';
import type { Client } from '../../clients/createClient.js';
import type { Transport } from '../../clients/transports/createTransport.js';
import { type ChainNotFoundErrorType } from '../../errors/chain.js';
import type { Chain } from '../../types/chain.js';
import type { Hash } from '../../types/misc.js';
import { type WithdrawalLogNotFoundErrorType } from '../errors/bridge.js';
import type { ChainEIP712 } from '../types/chain.js';
export type IsWithdrawalFinalizedParameters<chain extends Chain | undefined = Chain | undefined, account extends Account | undefined = Account | undefined> = {
    /** L2 client */
    client: Client<Transport, chain, account>;
    /** Hash of the L2 transaction where the withdrawal was initiated. */
    hash: Hash;
    /** In case there were multiple withdrawals in one transaction, you may pass an index of the
    withdrawal you want to finalize. */
    index?: number | undefined;
};
export type IsWithdrawalFinalizedReturnType = boolean;
export type IsWithdrawalFinalizedErrorType = WithdrawalLogNotFoundErrorType | ChainNotFoundErrorType;
/**
 * Returns whether the withdrawal transaction is finalized on the L1 network.
 *
 * @param client - Client to use
 * @param parameters - {@link IsWithdrawalFinalizedParameters}
 * @returns bool - Whether the withdrawal transaction is finalized on the L1 network. {@link IsWithdrawalFinalizedReturnType}
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { mainnet, zksync } from 'viem/chains'
 * import { isWithdrawalFinalized } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const clientL2 = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const hash = await isWithdrawalFinalized(client, {
 *     client: clientL2,
 *     hash: '0x...',
 * })
 */
export declare function isWithdrawalFinalized<chain extends Chain | undefined, account extends Account | undefined, chainL2 extends ChainEIP712 | undefined, accountL2 extends Account | undefined>(client: Client<Transport, chain, account>, parameters: IsWithdrawalFinalizedParameters<chainL2, accountL2>): Promise<IsWithdrawalFinalizedReturnType>;
//# sourceMappingURL=isWithdrawalFinalized.d.ts.map