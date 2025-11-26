import type * as Address from '../core/Address.js';
import * as Hex from '../core/Hex.js';
import * as Log from '../core/Log.js';
import * as TransactionReceipt from '../core/TransactionReceipt.js';
import type * as EntryPoint from './EntryPoint.js';
/**
 * User Operation Receipt type.
 *
 * @see https://eips.ethereum.org/EIPS/eip-4337#-eth_getuseroperationreceipt
 */
export type UserOperationReceipt<_entryPointVersion extends EntryPoint.Version = EntryPoint.Version, bigIntType = bigint, intType = number, receipt = TransactionReceipt.TransactionReceipt<TransactionReceipt.Status, TransactionReceipt.Type, bigIntType, intType>> = {
    /** Actual gas cost. */
    actualGasCost: bigIntType;
    /** Actual gas used. */
    actualGasUsed: bigIntType;
    /** Entrypoint address. */
    entryPoint: Address.Address;
    /** Logs emitted during execution. */
    logs: Log.Log<false, bigIntType, intType>[];
    /** Anti-replay parameter. */
    nonce: bigIntType;
    /** Paymaster for the user operation. */
    paymaster?: Address.Address | undefined;
    /** Revert reason, if unsuccessful. */
    reason?: string | undefined;
    /** Transaction receipt of the user operation execution. */
    receipt: receipt;
    /** The account sending the user operation. */
    sender: Address.Address;
    /** If the user operation execution was successful. */
    success: boolean;
    /** Hash of the user operation. */
    userOpHash: Hex.Hex;
};
/** RPC User Operation Receipt on EntryPoint 0.6 */
export type Rpc<entryPointVersion extends EntryPoint.Version = EntryPoint.Version> = UserOperationReceipt<entryPointVersion, Hex.Hex, Hex.Hex, TransactionReceipt.TransactionReceipt<TransactionReceipt.RpcStatus, TransactionReceipt.RpcType, Hex.Hex, Hex.Hex>>;
/**
 * Converts an {@link ox#UserOperationReceipt.Rpc} to an {@link ox#UserOperationReceipt.UserOperationReceipt}.
 *
 * @example
 * ```ts twoslash
 * // @noErrors
 * import { UserOperationReceipt } from 'ox/erc4337'
 *
 * const userOperationReceipt = UserOperationReceipt.fromRpc({
 *   actualGasCost: '0x1',
 *   actualGasUsed: '0x2',
 *   entryPoint: '0x0000000071727de22e5e9d8baf0edac6f37da032',
 *   logs: [],
 *   nonce: '0x1',
 *   receipt: { ... },
 *   sender: '0xE911628bF8428C23f179a07b081325cAe376DE1f',
 *   success: true,
 *   userOpHash: '0x5ab163e9b2f30549274c7c567ca0696edf9ef1aa476d9784d22974468fdb24d8',
 * })
 * ```
 *
 * @param rpc - The RPC user operation receipt to convert.
 * @returns An instantiated {@link ox#UserOperationReceipt.UserOperationReceipt}.
 */
export declare function fromRpc(rpc: Rpc): UserOperationReceipt;
/**
 * Converts a {@link ox#UserOperationReceipt.UserOperationReceipt} to a {@link ox#UserOperationReceipt.Rpc}.
 *
 * @example
 * ```ts twoslash
 * // @noErrors
 * import { UserOperationReceipt } from 'ox/erc4337'
 *
 * const userOperationReceipt = UserOperationReceipt.toRpc({
 *   actualGasCost: 1n,
 *   actualGasUsed: 2n,
 *   entryPoint: '0x0000000071727de22e5e9d8baf0edac6f37da032',
 *   logs: [],
 *   nonce: 1n,
 *   receipt: { ... },
 *   sender: '0xE911628bF8428C23f179a07b081325cAe376DE1f',
 *   success: true,
 *   userOpHash: '0x5ab163e9b2f30549274c7c567ca0696edf9ef1aa476d9784d22974468fdb24d8',
 * })
 * ```
 *
 * @param userOperationReceipt - The user operation receipt to convert.
 * @returns An RPC-formatted user operation receipt.
 */
export declare function toRpc(userOperationReceipt: UserOperationReceipt): Rpc;
//# sourceMappingURL=UserOperationReceipt.d.ts.map