import * as Hex from './Hex.js';
import * as Withdrawal from './Withdrawal.js';
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
export function fromRpc(rpcBlockOverrides) {
    return {
        ...(rpcBlockOverrides.baseFeePerGas && {
            baseFeePerGas: BigInt(rpcBlockOverrides.baseFeePerGas),
        }),
        ...(rpcBlockOverrides.blobBaseFee && {
            blobBaseFee: BigInt(rpcBlockOverrides.blobBaseFee),
        }),
        ...(rpcBlockOverrides.feeRecipient && {
            feeRecipient: rpcBlockOverrides.feeRecipient,
        }),
        ...(rpcBlockOverrides.gasLimit && {
            gasLimit: BigInt(rpcBlockOverrides.gasLimit),
        }),
        ...(rpcBlockOverrides.number && {
            number: BigInt(rpcBlockOverrides.number),
        }),
        ...(rpcBlockOverrides.prevRandao && {
            prevRandao: BigInt(rpcBlockOverrides.prevRandao),
        }),
        ...(rpcBlockOverrides.time && {
            time: BigInt(rpcBlockOverrides.time),
        }),
        ...(rpcBlockOverrides.withdrawals && {
            withdrawals: rpcBlockOverrides.withdrawals.map(Withdrawal.fromRpc),
        }),
    };
}
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
export function toRpc(blockOverrides) {
    return {
        ...(typeof blockOverrides.baseFeePerGas === 'bigint' && {
            baseFeePerGas: Hex.fromNumber(blockOverrides.baseFeePerGas),
        }),
        ...(typeof blockOverrides.blobBaseFee === 'bigint' && {
            blobBaseFee: Hex.fromNumber(blockOverrides.blobBaseFee),
        }),
        ...(typeof blockOverrides.feeRecipient === 'string' && {
            feeRecipient: blockOverrides.feeRecipient,
        }),
        ...(typeof blockOverrides.gasLimit === 'bigint' && {
            gasLimit: Hex.fromNumber(blockOverrides.gasLimit),
        }),
        ...(typeof blockOverrides.number === 'bigint' && {
            number: Hex.fromNumber(blockOverrides.number),
        }),
        ...(typeof blockOverrides.prevRandao === 'bigint' && {
            prevRandao: Hex.fromNumber(blockOverrides.prevRandao),
        }),
        ...(typeof blockOverrides.time === 'bigint' && {
            time: Hex.fromNumber(blockOverrides.time),
        }),
        ...(blockOverrides.withdrawals && {
            withdrawals: blockOverrides.withdrawals.map(Withdrawal.toRpc),
        }),
    };
}
//# sourceMappingURL=BlockOverrides.js.map