import { EventEmitter } from 'eventemitter3';
import type * as Address from './Address.js';
import * as Errors from './Errors.js';
import * as RpcResponse from './RpcResponse.js';
import type * as RpcSchema from './RpcSchema.js';
import type * as RpcSchema_internal from './internal/rpcSchema.js';
import type { Compute, IsNarrowable, IsNever } from './internal/types.js';
/** Options for a {@link ox#Provider.Provider}. */
export type Options = {
    /**
     * Whether to include event functions (`on`, `removeListener`) on the Provider.
     *
     * @default true
     */
    includeEvents?: boolean | undefined;
    /**
     * RPC Schema to use for the Provider's `request` function.
     * See {@link ox#RpcSchema.(from:function)} for more.
     *
     * @default `RpcSchema.Generic`
     */
    schema?: RpcSchema.Generic | undefined;
};
/** Root type for an EIP-1193 Provider. */
export type Provider<options extends Options | undefined = undefined, _schema extends RpcSchema.Generic = options extends {
    schema: infer schema extends RpcSchema.Generic;
} ? schema : RpcSchema.Default> = Compute<{
    request: RequestFn<_schema>;
} & (options extends {
    includeEvents: true;
} | undefined ? {
    on: EventListenerFn;
    removeListener: EventListenerFn;
} : {})>;
/** Type for an EIP-1193 Provider's event emitter. */
export type Emitter = Compute<EventEmitter<EventMap>>;
/** EIP-1193 Provider's `request` function. */
export type RequestFn<schema extends RpcSchema.Generic = RpcSchema.Generic> = <methodName extends RpcSchema.MethodNameGeneric>(parameters: RpcSchema_internal.ExtractRequestOpaque<schema, methodName>) => Promise<RpcSchema.ExtractReturnType<schema, methodName>>;
/** Type for an EIP-1193 Provider's event listener functions (`on`, `removeListener`, etc). */
export type EventListenerFn = <event extends keyof EventMap>(event: event, listener: EventMap[event]) => void;
export type ConnectInfo = {
    chainId: string;
};
export type Message = {
    type: string;
    data: unknown;
};
export declare class ProviderRpcError extends Error {
    name: string;
    code: number;
    details: string;
    constructor(code: number, message: string);
}
export type EventMap = {
    accountsChanged: (accounts: readonly Address.Address[]) => void;
    chainChanged: (chainId: string) => void;
    connect: (connectInfo: ConnectInfo) => void;
    disconnect: (error: ProviderRpcError) => void;
    message: (message: Message) => void;
};
/** The user rejected the request. */
export declare class UserRejectedRequestError extends ProviderRpcError {
    static readonly code = 4001;
    readonly code = 4001;
    readonly name = "Provider.UserRejectedRequestError";
    constructor({ message, }?: {
        message?: string | undefined;
    });
}
/** The requested method and/or account has not been authorized by the user. */
export declare class UnauthorizedError extends ProviderRpcError {
    static readonly code = 4100;
    readonly code = 4100;
    readonly name = "Provider.UnauthorizedError";
    constructor({ message, }?: {
        message?: string | undefined;
    });
}
/** The provider does not support the requested method. */
export declare class UnsupportedMethodError extends ProviderRpcError {
    static readonly code = 4200;
    readonly code = 4200;
    readonly name = "Provider.UnsupportedMethodError";
    constructor({ message, }?: {
        message?: string | undefined;
    });
}
/** The provider is disconnected from all chains. */
export declare class DisconnectedError extends ProviderRpcError {
    static readonly code = 4900;
    readonly code = 4900;
    readonly name = "Provider.DisconnectedError";
    constructor({ message, }?: {
        message?: string | undefined;
    });
}
/** The provider is not connected to the requested chain. */
export declare class ChainDisconnectedError extends ProviderRpcError {
    static readonly code = 4901;
    readonly code = 4901;
    readonly name = "Provider.ChainDisconnectedError";
    constructor({ message, }?: {
        message?: string | undefined;
    });
}
/**
 * Creates an EIP-1193 flavored event emitter to be injected onto a Provider.
 *
 * @example
 * ```ts twoslash
 * // @noErrors
 * import { Provider, RpcRequest, RpcResponse } from 'ox' // [!code focus]
 *
 * // 1. Instantiate a Provider Emitter. // [!code focus]
 * const emitter = Provider.createEmitter() // [!code focus]
 *
 * const store = RpcRequest.createStore()
 *
 * const provider = Provider.from({
 *   // 2. Pass the Emitter to the Provider. // [!code focus]
 *   ...emitter, // [!code focus]
 *   async request(args) {
 *     return await fetch('https://1.rpc.thirdweb.com', {
 *       body: JSON.stringify(store.prepare(args)),
 *       method: 'POST',
 *       headers: {
 *         'Content-Type': 'application/json',
 *       },
 *     })
 *       .then((res) => res.json())
 *       .then(RpcResponse.parse)
 *   },
 * })
 *
 * // 3. Emit Provider Events. // [!code focus]
 * emitter.emit('accountsChanged', ['0x...']) // [!code focus]
 * ```
 *
 * @returns An event emitter.
 */
export declare function createEmitter(): Emitter;
export declare namespace createEmitter {
    type ErrorType = Errors.GlobalErrorType;
}
/**
 * Instantiates an [EIP-1193](https://eips.ethereum.org/EIPS/eip-1193) {@link ox#Provider.Provider}
 * from an arbitrary [EIP-1193 Provider](https://eips.ethereum.org/EIPS/eip-1193) interface.
 *
 * @example
 * ### Instantiating with RPC Transport
 *
 * Ox's {@link ox#RpcTransport} is EIP-1193 compliant, and can be used to instantiate an EIP-1193 Provider. This means you can use any HTTP RPC endpoint as an EIP-1193 Provider.
 *
 * ```ts twoslash
 * import { Provider, RpcTransport } from 'ox'
 *
 * const transport = RpcTransport.fromHttp('https://1.rpc.thirdweb.com')
 * const provider = Provider.from(transport)
 * ```
 *
 * @example
 * ### Instantiating with External Providers
 *
 * The example below demonstrates how we can instantiate a typed EIP-1193 Provider from an
 * external EIP-1193 Provider like `window.ethereum`.
 *
 * ```ts twoslash
 * import 'ox/window'
 * import { Provider } from 'ox'
 *
 * const provider = Provider.from(window.ethereum)
 *
 * const blockNumber = await provider.request({ method: 'eth_blockNumber' })
 * ```
 *
 * :::tip
 *
 * There are also libraries that distribute EIP-1193 Provider objects that you can use with `Provider.from`:
 *
 * - [`@walletconnect/ethereum-provider`](https://www.npmjs.com/package/\@walletconnect/ethereum-provider)
 *
 * - [`@coinbase/wallet-sdk`](https://www.npmjs.com/package/\@coinbase/wallet-sdk)
 *
 * - [`@metamask/detect-provider`](https://www.npmjs.com/package/\@metamask/detect-provider)
 *
 * - [`@safe-global/safe-apps-provider`](https://github.com/safe-global/safe-apps-sdk/tree/main/packages/safe-apps-provider)
 *
 * - [`mipd`](https://github.com/wevm/mipd): EIP-6963 Multi Injected Providers
 *
 * :::
 *
 * @example
 * ### Instantiating a Custom Provider
 *
 * The example below demonstrates how we can instantiate a typed EIP-1193 Provider from a
 * HTTP `fetch` JSON-RPC request. You can use this pattern to integrate with any asynchronous JSON-RPC
 * transport, including WebSockets and IPC.
 *
 * ```ts twoslash
 * // @noErrors
 * import { Provider, RpcRequest, RpcResponse } from 'ox'
 *
 * const store = RpcRequest.createStore()
 *
 * const provider = Provider.from({
 *   async request(args) {
 *     return await fetch('https://1.rpc.thirdweb.com', {
 *       body: JSON.stringify(store.prepare(args)),
 *       method: 'POST',
 *       headers: {
 *         'Content-Type': 'application/json',
 *       },
 *     })
 *       .then((res) => res.json())
 *       .then(RpcResponse.parse)
 *   },
 * })
 *
 * const blockNumber = await provider.request({ method: 'eth_blockNumber' })
 * ```
 *
 * @example
 * ### Type-safe Custom Schemas
 *
 * It is possible to define your own type-safe schema by using the {@link ox#RpcSchema.(from:function)} type.
 *
 * ```ts twoslash
 * // @noErrors
 * import 'ox/window'
 * import { Provider, RpcSchema } from 'ox'
 *
 * const schema = RpcSchema.from<
 *   | RpcSchema.Default
 *   | {
 *       Request: {
 *         method: 'abe_foo',
 *         params: [id: number],
 *       }
 *       ReturnType: string
 *     }
 *   | {
 *       Request: {
 *         method: 'abe_bar',
 *         params: [id: string],
 *       }
 *       ReturnType: string
 *     }
 * >()
 *
 * const provider = Provider.from(window.ethereum, { schema })
 *
 * const blockNumber = await provider.request({ method: 'e' })
 * //                                                    ^|
 *
 *
 *
 *
 *
 * ```
 *
 * @example
 * ### Instantiating a Provider with Events
 *
 * The example below demonstrates how to instantiate a Provider with your own EIP-1193 flavored event emitter.
 *
 * This example is useful for Wallets that distribute an EIP-1193 Provider (e.g. webpage injection via `window.ethereum`).
 *
 * ```ts twoslash
 * // @noErrors
 * import { Provider, RpcRequest, RpcResponse } from 'ox'
 *
 * // 1. Instantiate a Provider Emitter.
 * const emitter = Provider.createEmitter() // [!code ++]
 *
 * const store = RpcRequest.createStore()
 *
 * const provider = Provider.from({
 *   // 2. Pass the Emitter to the Provider.
 *   ...emitter, // [!code ++]
 *   async request(args) {
 *     return await fetch('https://1.rpc.thirdweb.com', {
 *       body: JSON.stringify(store.prepare(args)),
 *       method: 'POST',
 *       headers: {
 *         'Content-Type': 'application/json',
 *       },
 *     })
 *       .then((res) => res.json())
 *       .then(RpcResponse.parse)
 *   },
 * })
 *
 * // 3. Emit Provider Events.
 * emitter.emit('accountsChanged', ['0x...']) // [!code ++]
 * ```
 *
 * @param provider - The EIP-1193 provider to convert.
 * @returns An typed EIP-1193 Provider.
 */
export declare function from<const provider extends Provider | unknown, options extends Options | undefined = undefined>(provider: provider | Provider<{
    schema: RpcSchema.Generic;
}>, options?: options | Options): Provider<options>;
export declare namespace from {
    type ErrorType = IsUndefinedError | Errors.GlobalErrorType;
}
/**
 * Parses an error object into an error instance.
 *
 * @example
 * ```ts twoslash
 * import { Provider } from 'ox'
 *
 * const error = Provider.parseError({ code: 4200, message: 'foo' })
 *
 * error
 * // ^?
 *
 * ```
 *
 * @param errorObject - The error object to parse.
 * @returns An error instance.
 */
export declare function parseError<const errorObject extends RpcResponse.ErrorObject | unknown>(errorObject: errorObject | RpcResponse.ErrorObject): parseError.ReturnType<errorObject>;
export declare namespace parseError {
    type ReturnType<errorObject extends RpcResponse.ErrorObject | unknown, error = errorObject extends RpcResponse.ErrorObject ? (errorObject['code'] extends DisconnectedError['code'] ? DisconnectedError : never) | (IsNarrowable<errorObject['code'], number> extends false ? DisconnectedError : never) | (errorObject['code'] extends ChainDisconnectedError['code'] ? ChainDisconnectedError : never) | (IsNarrowable<errorObject['code'], number> extends false ? ChainDisconnectedError : never) | (errorObject['code'] extends UserRejectedRequestError['code'] ? UserRejectedRequestError : never) | (IsNarrowable<errorObject['code'], number> extends false ? UserRejectedRequestError : never) | (errorObject['code'] extends UnauthorizedError['code'] ? UnauthorizedError : never) | (IsNarrowable<errorObject['code'], number> extends false ? UnauthorizedError : never) | (errorObject['code'] extends UnsupportedMethodError['code'] ? UnsupportedMethodError : never) | (IsNarrowable<errorObject['code'], number> extends false ? UnsupportedMethodError : never) : RpcResponse.parseError.ReturnType<RpcResponse.ErrorObject>> = IsNever<error> extends true ? RpcResponse.parseError.ReturnType<errorObject> : error;
}
/** Thrown when the provider is undefined. */
export declare class IsUndefinedError extends Errors.BaseError {
    readonly name = "Provider.IsUndefinedError";
    constructor();
}
//# sourceMappingURL=Provider.d.ts.map