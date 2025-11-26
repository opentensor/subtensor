import type * as Address from './Address.js'
import * as Hex from './Hex.js'
import type { Compute, OneOf } from './internal/types.js'

/**
 * State override set to specify state to be ephemerally overridden prior to executing a call.
 */
export type StateOverrides<bigintType = bigint> = Compute<{
  [address: Address.Address]: AccountOverrides<bigintType>
}>

/**
 * RPC state overrides.
 */
export type Rpc = StateOverrides<Hex.Hex>

/**
 * Details of an account to be overridden.
 */
export type AccountOverrides<bigintType = bigint> = Compute<
  {
    /** Balance to set for the account. */
    balance?: bigintType | undefined
    /** Code to set for the account. */
    code?: Hex.Hex | undefined
    /** Address to move the precompile to. */
    movePrecompileToAddress?: Address.Address | undefined
    /** Nonce to set for the account. */
    nonce?: bigintType | undefined
  } & OneOf<
    | {
        /** Key-value mapping to override all slots in the account storage. */
        state?: AccountStorage | undefined
      }
    | {
        /** Key-value mapping to override individual slots in the account storage. */
        stateDiff?: AccountStorage | undefined
      }
  >
>

/**
 * RPC account overrides.
 */
export type RpcAccountOverrides = AccountOverrides<Hex.Hex>

/**
 * Key-value mapping to override all slots in the account storage before executing the call.
 */
export type AccountStorage = Compute<{
  [slot: Hex.Hex]: Hex.Hex
}>

/**
 * Converts an {@link ox#StateOverrides.Rpc} to an {@link ox#StateOverrides.StateOverrides}.
 *
 * @example
 * ```ts twoslash
 * import { StateOverrides } from 'ox'
 *
 * const stateOverrides = StateOverrides.fromRpc({
 *   '0x0000000000000000000000000000000000000000': {
 *     balance: '0x1',
 *   },
 * })
 * ```
 *
 * @param rpcStateOverrides - The RPC state overrides to convert.
 * @returns An instantiated {@link ox#StateOverrides.StateOverrides}.
 */
export function fromRpc(rpcStateOverrides: Rpc): StateOverrides {
  const stateOverrides: StateOverrides = {}
  for (const [address, accountOverridesRpc] of Object.entries(
    rpcStateOverrides,
  )) {
    const accountOverrides: AccountOverrides = {}
    if (accountOverridesRpc.balance)
      accountOverrides.balance = BigInt(accountOverridesRpc.balance)
    if (accountOverridesRpc.code)
      accountOverrides.code = accountOverridesRpc.code
    if (accountOverridesRpc.movePrecompileToAddress)
      accountOverrides.movePrecompileToAddress =
        accountOverridesRpc.movePrecompileToAddress
    if (accountOverridesRpc.nonce)
      accountOverrides.nonce = BigInt(accountOverridesRpc.nonce)
    if (accountOverridesRpc.state)
      accountOverrides.state = accountOverridesRpc.state
    if (accountOverridesRpc.stateDiff)
      accountOverrides.stateDiff = accountOverridesRpc.stateDiff
    ;(stateOverrides as any)[address] = accountOverrides
  }
  return stateOverrides
}

/**
 * Converts an {@link ox#StateOverrides.StateOverrides} to an {@link ox#StateOverrides.Rpc}.
 *
 * @example
 * ```ts twoslash
 * import { StateOverrides } from 'ox'
 *
 * const stateOverrides = StateOverrides.toRpc({
 *   '0x0000000000000000000000000000000000000000': {
 *     balance: 1n,
 *   },
 * })
 * ```
 *
 * @param stateOverrides - The state overrides to convert.
 * @returns An instantiated {@link ox#StateOverrides.Rpc}.
 */
export function toRpc(stateOverrides: StateOverrides): Rpc {
  const rpcStateOverrides: Rpc = {}
  for (const [address, accountOverrides] of Object.entries(stateOverrides)) {
    const accountOverridesRpc: RpcAccountOverrides = {}
    if (typeof accountOverrides.balance === 'bigint')
      accountOverridesRpc.balance = Hex.fromNumber(accountOverrides.balance)
    if (accountOverrides.code) accountOverridesRpc.code = accountOverrides.code
    if (accountOverrides.movePrecompileToAddress)
      accountOverridesRpc.movePrecompileToAddress =
        accountOverrides.movePrecompileToAddress
    if (typeof accountOverrides.nonce === 'bigint')
      accountOverridesRpc.nonce = Hex.fromNumber(accountOverrides.nonce)
    if (accountOverrides.state)
      accountOverridesRpc.state = accountOverrides.state
    if (accountOverrides.stateDiff)
      accountOverridesRpc.stateDiff = accountOverrides.stateDiff
    ;(rpcStateOverrides as any)[address] = accountOverridesRpc
  }
  return rpcStateOverrides
}
