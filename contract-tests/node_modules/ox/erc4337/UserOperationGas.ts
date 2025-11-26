import * as Hex from '../core/Hex.js'
import type { OneOf } from '../core/internal/types.js'
import type * as EntryPoint from './EntryPoint.js'

/** User Operation Gas type. */
export type UserOperationGas<
  entryPointVersion extends EntryPoint.Version = EntryPoint.Version,
  bigintType = bigint,
> = OneOf<
  | (entryPointVersion extends '0.6' ? V06<bigintType> : never)
  | (entryPointVersion extends '0.7' ? V07<bigintType> : never)
>

/** RPC User Operation Gas on EntryPoint 0.6 */
export type Rpc<
  entryPointVersion extends EntryPoint.Version = EntryPoint.Version,
> = UserOperationGas<entryPointVersion, Hex.Hex>

/** Type for User Operation Gas on EntryPoint 0.6 */
export type V06<bigintType = bigint> = {
  callGasLimit: bigintType
  preVerificationGas: bigintType
  verificationGasLimit: bigintType
}

/** RPC User Operation Gas on EntryPoint 0.6 */
export type RpcV06 = V06<Hex.Hex>

/** Type for User Operation Gas on EntryPoint 0.7 */
export type V07<bigintType = bigint> = {
  callGasLimit: bigintType
  paymasterVerificationGasLimit?: bigintType | undefined
  paymasterPostOpGasLimit?: bigintType | undefined
  preVerificationGas: bigintType
  verificationGasLimit: bigintType
}

/** RPC User Operation Gas on EntryPoint 0.7 */
export type RpcV07 = V07<Hex.Hex>

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
export function fromRpc(rpc: Rpc): UserOperationGas {
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
  } as UserOperationGas
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
export function toRpc(userOperationGas: UserOperationGas): Rpc {
  const rpc = {} as Rpc

  rpc.callGasLimit = Hex.fromNumber(userOperationGas.callGasLimit)
  rpc.preVerificationGas = Hex.fromNumber(userOperationGas.preVerificationGas)
  rpc.verificationGasLimit = Hex.fromNumber(
    userOperationGas.verificationGasLimit,
  )

  if (typeof userOperationGas.paymasterVerificationGasLimit === 'bigint')
    rpc.paymasterVerificationGasLimit = Hex.fromNumber(
      userOperationGas.paymasterVerificationGasLimit,
    )
  if (typeof userOperationGas.paymasterPostOpGasLimit === 'bigint')
    rpc.paymasterPostOpGasLimit = Hex.fromNumber(
      userOperationGas.paymasterPostOpGasLimit,
    )

  return rpc
}
