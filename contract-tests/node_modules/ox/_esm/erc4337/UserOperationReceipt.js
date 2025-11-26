import * as Hex from '../core/Hex.js';
import * as Log from '../core/Log.js';
import * as TransactionReceipt from '../core/TransactionReceipt.js';
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
export function fromRpc(rpc) {
    return {
        ...rpc,
        actualGasCost: BigInt(rpc.actualGasCost),
        actualGasUsed: BigInt(rpc.actualGasUsed),
        logs: rpc.logs.map((log) => Log.fromRpc(log)),
        nonce: BigInt(rpc.nonce),
        receipt: TransactionReceipt.fromRpc(rpc.receipt),
    };
}
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
export function toRpc(userOperationReceipt) {
    const rpc = {};
    rpc.actualGasCost = Hex.fromNumber(userOperationReceipt.actualGasCost);
    rpc.actualGasUsed = Hex.fromNumber(userOperationReceipt.actualGasUsed);
    rpc.entryPoint = userOperationReceipt.entryPoint;
    rpc.logs = userOperationReceipt.logs.map((log) => Log.toRpc(log));
    rpc.nonce = Hex.fromNumber(userOperationReceipt.nonce);
    rpc.receipt = TransactionReceipt.toRpc(userOperationReceipt.receipt);
    rpc.sender = userOperationReceipt.sender;
    rpc.success = userOperationReceipt.success;
    rpc.userOpHash = userOperationReceipt.userOpHash;
    if (userOperationReceipt.paymaster)
        rpc.paymaster = userOperationReceipt.paymaster;
    if (userOperationReceipt.reason)
        rpc.reason = userOperationReceipt.reason;
    return rpc;
}
//# sourceMappingURL=UserOperationReceipt.js.map