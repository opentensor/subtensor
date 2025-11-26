import type * as Address from './Address.js';
import * as Hex from './Hex.js';
import * as Withdrawal from './Withdrawal.js';
/**
 * Block overrides.
 */
export type BlockOverrides<bigintType = bigint, numberType = number> = {
    /** Base fee per gas. */
    baseFeePerGas?: bigintType | undefined;
    /** Blob base fee. */
    blobBaseFee?: bigintType | undefined;
    /** Fee recipient (also known as coinbase). */
    feeRecipient?: Address.Address | undefined;
    /** Gas limit. */
    gasLimit?: bigintType | undefined;
    /** Block number. */
    number?: bigintType | undefined;
    /** The previous value of randomness beacon. */
    prevRandao?: bigintType | undefined;
    /** Block timestamp. */
    time?: bigintType | undefined;
    /** Withdrawals made by validators. */
    withdrawals?: Withdrawal.Withdrawal<bigintType, numberType>[] | undefined;
};
/**
 * RPC block overrides.
 */
export type Rpc = BlockOverrides<Hex.Hex, Hex.Hex>;
/**
 * Converts an {@link ox#BlockOverrides.Rpc} to an {@link ox#BlockOverrides.BlockOverrides}.
 *
 * @example
 * ```ts twoslash
 * import { BlockOverrides } from 'ox'
 *
 * const blockOverrides = BlockOverrides.fromRpc({
 *   baseFeePerGas: '0x1',
 *   blobBaseFee: '0x2',
 *   feeRecipient: '0x0000000000000000000000000000000000000000',
 *   gasLimit: '0x4',
 *   number: '0x5',
 *   prevRandao: '0x6',
 *   time: '0x1234567890',
 *   withdrawals: [
 *     {
 *       address: '0x0000000000000000000000000000000000000000',
 *       amount: '0x1',
 *       index: '0x0',
 *       validatorIndex: '0x1',
 *     },
 *   ],
 * })
 * ```
 *
 * @param rpcBlockOverrides - The RPC block overrides to convert.
 * @returns An instantiated {@link ox#BlockOverrides.BlockOverrides}.
 */
export declare function fromRpc(rpcBlockOverrides: Rpc): BlockOverrides;
/**
 * Converts an {@link ox#BlockOverrides.BlockOverrides} to an {@link ox#BlockOverrides.Rpc}.
 *
 * @example
 * ```ts twoslash
 * import { BlockOverrides } from 'ox'
 *
 * const blockOverrides = BlockOverrides.toRpc({
 *   baseFeePerGas: 1n,
 *   blobBaseFee: 2n,
 *   feeRecipient: '0x0000000000000000000000000000000000000000',
 *   gasLimit: 4n,
 *   number: 5n,
 *   prevRandao: 6n,
 *   time: 78187493520n,
 *   withdrawals: [
 *     {
 *       address: '0x0000000000000000000000000000000000000000',
 *       amount: 1n,
 *       index: 0,
 *       validatorIndex: 1,
 *     },
 *   ],
 * })
 * ```
 *
 * @param blockOverrides - The block overrides to convert.
 * @returns An instantiated {@link ox#BlockOverrides.Rpc}.
 */
export declare function toRpc(blockOverrides: BlockOverrides): Rpc;
//# sourceMappingURL=BlockOverrides.d.ts.map