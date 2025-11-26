import type * as Address from '../core/Address.js';
import type * as Hex from '../core/Hex.js';
import type * as RpcSchema from '../core/RpcSchema.js';
import type * as StateOverrides from '../core/StateOverrides.js';
import type * as EntryPoint from './EntryPoint.js';
import type * as UserOperation from './UserOperation.js';
import type * as UserOperationGas from './UserOperationGas.js';
import type * as UserOperationReceipt from './UserOperationReceipt.js';
/**
 * Union of all JSON-RPC Methods for ERC-4337 Bundlers.
 *
 * @example
 * ```ts twoslash
 * import { RpcSchema } from 'ox'
 *
 * type Schema = RpcSchema.Bundler
 * //   ^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 */
export type Bundler<entryPointVersion extends EntryPoint.Version = EntryPoint.Version> = RpcSchema.From<{
    Request: {
        method: 'eth_chainId';
        params?: undefined;
    };
    ReturnType: Hex.Hex;
} | {
    Request: {
        method: 'eth_estimateUserOperationGas';
        params: [
            userOperation: UserOperation.Rpc<entryPointVersion>,
            entrypoint: Address.Address
        ] | [
            userOperation: UserOperation.Rpc<entryPointVersion>,
            entrypoint: Address.Address,
            stateOverrides: StateOverrides.Rpc
        ];
    };
    ReturnType: UserOperationGas.Rpc<entryPointVersion>;
} | {
    Request: {
        method: 'eth_getUserOperationByHash';
        params: [hash: Hex.Hex];
    };
    ReturnType: UserOperation.Rpc<entryPointVersion> | null;
} | {
    Request: {
        method: 'eth_getUserOperationReceipt';
        params: [hash: Hex.Hex];
    };
    ReturnType: UserOperationReceipt.Rpc<entryPointVersion> | null;
} | {
    Request: {
        method: 'eth_sendUserOperation';
        params: [
            userOperation: UserOperation.Rpc<entryPointVersion>,
            entrypoint: Address.Address
        ];
    };
    ReturnType: Hex.Hex;
} | {
    Request: {
        method: 'eth_supportedEntryPoints';
        params?: undefined;
    };
    ReturnType: readonly Address.Address[];
}>;
/**
 * Union of all JSON-RPC Methods for the debug methods of ERC-4337 Bundlers.
 *
 * @example
 * ```ts twoslash
 * import { RpcSchema } from 'ox'
 *
 * type Schema = RpcSchema.BundlerDebug
 * //   ^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 */
export type BundlerDebug<entryPointVersion extends EntryPoint.Version = EntryPoint.Version> = RpcSchema.From<{
    Request: {
        method: 'debug_bundler_clearState';
        params?: undefined;
    };
    ReturnType: undefined;
} | {
    Request: {
        method: 'debug_bundler_dumpMempool';
        params: [entryPoint: Address.Address];
    };
    ReturnType: readonly {
        userOp: UserOperation.Rpc;
    }[];
} | {
    Request: {
        method: 'debug_bundler_sendBundleNow';
        params?: undefined;
    };
    ReturnType: Hex.Hex;
} | {
    Request: {
        method: 'debug_bundler_setBundlingMode';
        params: [mode: 'auto' | 'manual'];
    };
    ReturnType: undefined;
} | {
    Request: {
        method: 'debug_bundler_setReputation';
        params: [
            reputations: readonly {
                address: Address.Address;
                opsSeen: Hex.Hex;
                opsIncluded: Hex.Hex;
            }[],
            entryPoint: Address.Address
        ];
    };
    ReturnType: undefined;
} | {
    Request: {
        method: 'debug_bundler_dumpReputation';
        params: [entryPoint: Address.Address];
    };
    ReturnType: readonly {
        address: Address.Address;
        opsSeen: Hex.Hex;
        opsIncluded: Hex.Hex;
    }[];
} | {
    Request: {
        method: 'debug_bundler_addUserOps';
        params: [
            userOps: readonly UserOperation.Rpc<entryPointVersion>[],
            entryPoint: Address.Address
        ];
    };
    ReturnType: undefined;
}>;
//# sourceMappingURL=RpcSchema.d.ts.map