import type { Address } from 'abitype';
import type { TransactionReceipt } from '../../../types/transaction.js';
import { type TxHashNotFoundInLogsErrorType } from '../../errors/bridge.js';
export type GetL2HashFromPriorityOpErrorType = TxHashNotFoundInLogsErrorType;
/**
 * Returns the hash of the L2 priority operation from a given L1 transaction receipt.
 *
 * @param receipt - The L1 transaction receipt.
 * @param zksync - The address of the ZKsync Era main contract.
 * @returns hash - The hash of the L2 priority operation.
 *
 * @example
 * import { createPublicClient, http } from 'viem'
 * import { zksync, mainnet } from 'viem/chains'
 * import { publicActionsL2, getL2HashFromPriorityOp } from 'viem/zksync'
 *
 * const client = createPublicClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const zksyncClient = const client = createPublicClient({
 *   chain: zksync,
 *   transport: http(),
 * })
 *
 * const receipt = await client.waitForTransactionReceipt({hash: '0x...'})
 * const l2Hash = getL2HashFromPriorityOp(
 *   receipt,
 *   await zksyncClient.getMainContractAddress()
 * )
 */
export declare function getL2HashFromPriorityOp(receipt: TransactionReceipt, zksync: Address): Address;
//# sourceMappingURL=getL2HashFromPriorityOp.d.ts.map