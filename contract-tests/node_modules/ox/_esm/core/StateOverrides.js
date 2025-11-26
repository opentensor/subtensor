import * as Hex from './Hex.js';
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
export function fromRpc(rpcStateOverrides) {
    const stateOverrides = {};
    for (const [address, accountOverridesRpc] of Object.entries(rpcStateOverrides)) {
        const accountOverrides = {};
        if (accountOverridesRpc.balance)
            accountOverrides.balance = BigInt(accountOverridesRpc.balance);
        if (accountOverridesRpc.code)
            accountOverrides.code = accountOverridesRpc.code;
        if (accountOverridesRpc.movePrecompileToAddress)
            accountOverrides.movePrecompileToAddress =
                accountOverridesRpc.movePrecompileToAddress;
        if (accountOverridesRpc.nonce)
            accountOverrides.nonce = BigInt(accountOverridesRpc.nonce);
        if (accountOverridesRpc.state)
            accountOverrides.state = accountOverridesRpc.state;
        if (accountOverridesRpc.stateDiff)
            accountOverrides.stateDiff = accountOverridesRpc.stateDiff;
        stateOverrides[address] = accountOverrides;
    }
    return stateOverrides;
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
export function toRpc(stateOverrides) {
    const rpcStateOverrides = {};
    for (const [address, accountOverrides] of Object.entries(stateOverrides)) {
        const accountOverridesRpc = {};
        if (typeof accountOverrides.balance === 'bigint')
            accountOverridesRpc.balance = Hex.fromNumber(accountOverrides.balance);
        if (accountOverrides.code)
            accountOverridesRpc.code = accountOverrides.code;
        if (accountOverrides.movePrecompileToAddress)
            accountOverridesRpc.movePrecompileToAddress =
                accountOverrides.movePrecompileToAddress;
        if (typeof accountOverrides.nonce === 'bigint')
            accountOverridesRpc.nonce = Hex.fromNumber(accountOverrides.nonce);
        if (accountOverrides.state)
            accountOverridesRpc.state = accountOverrides.state;
        if (accountOverrides.stateDiff)
            accountOverridesRpc.stateDiff = accountOverrides.stateDiff;
        rpcStateOverrides[address] = accountOverridesRpc;
    }
    return rpcStateOverrides;
}
//# sourceMappingURL=StateOverrides.js.map