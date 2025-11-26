import * as Hex from '../core/Hex.js';
/**
 * Converts an {@link ox#UserOperationGas.Rpc} to an {@link ox#UserOperationGas.UserOperationGas}.
 *
 * @example
 * ```ts twoslash
 * import { UserOperationGas } from 'ox/erc4337'
 *
 * const userOperationGas = UserOperationGas.fromRpc({
 *   callGasLimit: '0x69420',
 *   preVerificationGas: '0x69420',
 *   verificationGasLimit: '0x69420',
 * })
 * ```
 *
 * @param rpc - The RPC user operation gas to convert.
 * @returns An instantiated {@link ox#UserOperationGas.UserOperationGas}.
 */
export function fromRpc(rpc) {
    return {
        ...rpc,
        callGasLimit: BigInt(rpc.callGasLimit),
        preVerificationGas: BigInt(rpc.preVerificationGas),
        verificationGasLimit: BigInt(rpc.verificationGasLimit),
        ...(rpc.paymasterVerificationGasLimit && {
            paymasterVerificationGasLimit: BigInt(rpc.paymasterVerificationGasLimit),
        }),
        ...(rpc.paymasterPostOpGasLimit && {
            paymasterPostOpGasLimit: BigInt(rpc.paymasterPostOpGasLimit),
        }),
    };
}
/**
 * Converts a {@link ox#UserOperationGas.UserOperationGas} to a {@link ox#UserOperationGas.Rpc}.
 *
 * @example
 * ```ts twoslash
 * import { UserOperationGas } from 'ox/erc4337'
 *
 * const userOperationGas = UserOperationGas.toRpc({
 *   callGasLimit: 300_000n,
 *   preVerificationGas: 100_000n,
 *   verificationGasLimit: 100_000n,
 * })
 * ```
 *
 * @param userOperationGas - The user operation gas to convert.
 * @returns An RPC-formatted user operation gas.
 */
export function toRpc(userOperationGas) {
    const rpc = {};
    rpc.callGasLimit = Hex.fromNumber(userOperationGas.callGasLimit);
    rpc.preVerificationGas = Hex.fromNumber(userOperationGas.preVerificationGas);
    rpc.verificationGasLimit = Hex.fromNumber(userOperationGas.verificationGasLimit);
    if (typeof userOperationGas.paymasterVerificationGasLimit === 'bigint')
        rpc.paymasterVerificationGasLimit = Hex.fromNumber(userOperationGas.paymasterVerificationGasLimit);
    if (typeof userOperationGas.paymasterPostOpGasLimit === 'bigint')
        rpc.paymasterPostOpGasLimit = Hex.fromNumber(userOperationGas.paymasterPostOpGasLimit);
    return rpc;
}
//# sourceMappingURL=UserOperationGas.js.map