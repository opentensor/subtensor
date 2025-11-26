import { EventEmitter } from 'eventemitter3'
import type * as Address from './Address.js'
import * as Errors from './Errors.js'
import type * as RpcSchema_internal from './internal/rpcSchema.js'
import type { Compute, IsNarrowable, IsNever } from './internal/types.js'
import * as RpcResponse from './RpcResponse.js'
import type * as RpcSchema from './RpcSchema.js'

/** Options for a {@link ox#Provider.Provider}. */
export type Options = {
  /**
   * Whether to include event functions (`on`, `removeListener`) on the Provider.
   *
   * @default true
   */
  includeEvents?: boolean | undefined
  /**
   * RPC Schema to use for the Provider's `request` function.
   * See {@link ox#RpcSchema.(from:function)} for more.
   *
   * @default `RpcSchema.Generic`
   */
  schema?: RpcSchema.Generic | undefined
}

/** Root type for an EIP-1193 Provider. */
export type Provider<
  options extends Options | undefined = undefined,
  ///
  _schema extends RpcSchema.Generic = options extends {
    schema: infer schema extends RpcSchema.Generic
  }
    ? schema
    : RpcSchema.Default,
> = Compute<
  {
    request: RequestFn<_schema>
  } & (options extends { includeEvents: true } | undefined
    ? {
        on: EventListenerFn
        removeListener: EventListenerFn
      }
    : {})
>

/** Type for an EIP-1193 Provider's event emitter. */
export type Emitter = Compute<EventEmitter<EventMap>>

/** EIP-1193 Provider's `request` function. */
export type RequestFn<schema extends RpcSchema.Generic = RpcSchema.Generic> = <
  methodName extends RpcSchema.MethodNameGeneric,
>(
  parameters: RpcSchema_internal.ExtractRequestOpaque<schema, methodName>,
) => Promise<RpcSchema.ExtractReturnType<schema, methodName>>

/** Type for an EIP-1193 Provider's event listener functions (`on`, `removeListener`, etc). */
export type EventListenerFn = <event extends keyof EventMap>(
  event: event,
  listener: EventMap[event],
) => void

export type ConnectInfo = {
  chainId: string
}

export type Message = {
  type: string
  data: unknown
}

export class ProviderRpcError extends Error {
  override name = 'ProviderRpcError'

  code: number
  details: string

  constructor(code: number, message: string) {
    super(message)
    this.code = code
    this.details = message
  }
}

export type EventMap = {
  accountsChanged: (accounts: readonly Address.Address[]) => void
  chainChanged: (chainId: string) => void
  connect: (connectInfo: ConnectInfo) => void
  disconnect: (error: ProviderRpcError) => void
  message: (message: Message) => void
}

/** The user rejected the request. */
export class UserRejectedRequestError extends ProviderRpcError {
  static readonly code = 4001
  override readonly code = 4001
  override readonly name = 'Provider.UserRejectedRequestError'

  constructor({
    message = 'The user rejected the request.',
  }: { message?: string | undefined } = {}) {
    super(4001, message)
  }
}

/** The requested method and/or account has not been authorized by the user. */
export class UnauthorizedError extends ProviderRpcError {
  static readonly code = 4100
  override readonly code = 4100
  override readonly name = 'Provider.UnauthorizedError'

  constructor({
    message = 'The requested method and/or account has not been authorized by the user.',
  }: { message?: string | undefined } = {}) {
    super(4100, message)
  }
}

/** The provider does not support the requested method. */
export class UnsupportedMethodError extends ProviderRpcError {
  static readonly code = 4200
  override readonly code = 4200
  override readonly name = 'Provider.UnsupportedMethodError'

  constructor({
    message = 'The provider does not support the requested method.',
  }: { message?: string | undefined } = {}) {
    super(4200, message)
  }
}

/** The provider is disconnected from all chains. */
export class DisconnectedError extends ProviderRpcError {
  static readonly code = 4900
  override readonly code = 4900
  override readonly name = 'Provider.DisconnectedError'

  constructor({
    message = 'The provider is disconnected from all chains.',
  }: { message?: string | undefined } = {}) {
    super(4900, message)
  }
}

/** The provider is not connected to the requested chain. */
export class ChainDisconnectedError extends ProviderRpcError {
  static readonly code = 4901
  override readonly code = 4901
  override readonly name = 'Provider.ChainDisconnectedError'

  constructor({
    message = 'The provider is not connected to the requested chain.',
  }: { message?: string | undefined } = {}) {
    super(4901, message)
  }
}

/** An error occurred when attempting to switch chain. */
export class SwitchChainError extends ProviderRpcError {
  static readonly code = 4902
  override readonly code = 4902
  override readonly name = 'Provider.SwitchChainError'

  constructor({
    message = 'An error occurred when attempting to switch chain.',
  }: { message?: string | undefined } = {}) {
    super(4902, message)
  }
}

/** This Wallet does not support a capability that was not marked as optional. */
export class UnsupportedNonOptionalCapabilityError extends ProviderRpcError {
  static readonly code = 5700
  override readonly code = 5700
  override readonly name = 'Provider.UnsupportedNonOptionalCapabilityError'

  constructor({
    message = 'This Wallet does not support a capability that was not marked as optional.',
  }: { message?: string | undefined } = {}) {
    super(5700, message)
  }
}

/** This Wallet does not support the requested chain ID. */
export class UnsupportedChainIdError extends ProviderRpcError {
  static readonly code = 5710
  override readonly code = 5710
  override readonly name = 'Provider.UnsupportedChainIdError'

  constructor({
    message = 'This Wallet does not support the requested chain ID.',
  }: { message?: string | undefined } = {}) {
    super(5710, message)
  }
}

/** There is already a bundle submitted with this ID. */
export class DuplicateIdError extends ProviderRpcError {
  static readonly code = 5720
  override readonly code = 5720
  override readonly name = 'Provider.DuplicateIdError'

  constructor({
    message = 'There is already a bundle submitted with this ID.',
  }: { message?: string | undefined } = {}) {
    super(5720, message)
  }
}

/** This bundle id is unknown / has not been submitted. */
export class UnknownBundleIdError extends ProviderRpcError {
  static readonly code = 5730
  override readonly code = 5730
  override readonly name = 'Provider.UnknownBundleIdError'

  constructor({
    message = 'This bundle id is unknown / has not been submitted.',
  }: { message?: string | undefined } = {}) {
    super(5730, message)
  }
}

/** The call bundle is too large for the Wallet to process. */
export class BundleTooLargeError extends ProviderRpcError {
  static readonly code = 5740
  override readonly code = 5740
  override readonly name = 'Provider.BundleTooLargeError'

  constructor({
    message = 'The call bundle is too large for the Wallet to process.',
  }: { message?: string | undefined } = {}) {
    super(5740, message)
  }
}

/** The Wallet can support atomicity after an upgrade, but the user rejected the upgrade. */
export class AtomicReadyWalletRejectedUpgradeError extends ProviderRpcError {
  static readonly code = 5750
  override readonly code = 5750
  override readonly name = 'Provider.AtomicReadyWalletRejectedUpgradeError'

  constructor({
    message = 'The Wallet can support atomicity after an upgrade, but the user rejected the upgrade.',
  }: { message?: string | undefined } = {}) {
    super(5750, message)
  }
}

/** The wallet does not support atomic execution but the request requires it. */
export class AtomicityNotSupportedError extends ProviderRpcError {
  static readonly code = 5760
  override readonly code = 5760
  override readonly name = 'Provider.AtomicityNotSupportedError'

  constructor({
    message = 'The wallet does not support atomic execution but the request requires it.',
  }: { message?: string | undefined } = {}) {
    super(5760, message)
  }
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
export function createEmitter(): Emitter {
  const emitter = new EventEmitter<EventMap>()

  return {
    get eventNames() {
      return emitter.eventNames.bind(emitter)
    },
    get listenerCount() {
      return emitter.listenerCount.bind(emitter)
    },
    get listeners() {
      return emitter.listeners.bind(emitter)
    },
    addListener: emitter.addListener.bind(emitter),
    emit: emitter.emit.bind(emitter),
    off: emitter.off.bind(emitter),
    on: emitter.on.bind(emitter),
    once: emitter.once.bind(emitter),
    removeAllListeners: emitter.removeAllListeners.bind(emitter),
    removeListener: emitter.removeListener.bind(emitter),
  }
}

export declare namespace createEmitter {
  type ErrorType = Errors.GlobalErrorType
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
export function from<
  const provider extends Provider | unknown,
  options extends Options | undefined = undefined,
>(
  provider: provider | Provider<{ schema: RpcSchema.Generic }>,
  options?: options | Options,
): Provider<options>
// eslint-disable-next-line jsdoc/require-jsdoc
export function from(provider: any, options: Options = {}): Provider<Options> {
  const { includeEvents = true } = options
  if (!provider) throw new IsUndefinedError()
  return {
    ...(includeEvents
      ? {
          on: provider.on?.bind(provider),
          removeListener: provider.removeListener?.bind(provider),
        }
      : {}),
    async request(args) {
      try {
        const result = await provider.request(args)
        if (
          result &&
          typeof result === 'object' &&
          'jsonrpc' in (result as { jsonrpc?: unknown })
        )
          return RpcResponse.parse(result) as never
        return result
      } catch (error) {
        throw parseError(error)
      }
    },
  }
}

export declare namespace from {
  type ErrorType = IsUndefinedError | Errors.GlobalErrorType
}

/**
 * Parses an error into a Provider error instance.
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
 * @param error - The error object to parse.
 * @returns An error instance.
 */
export function parseError<
  const error extends RpcResponse.ErrorObject | Error | unknown,
>(
  error: error | Error | RpcResponse.ErrorObject,
): parseError.ReturnType<error> {
  const error_ = RpcResponse.parseError(error)
  if (error_ instanceof RpcResponse.InternalError) {
    if (!error_.data) return error_ as never

    const { code } = error_.data as RpcResponse.ErrorObject
    if (code === DisconnectedError.code)
      return new DisconnectedError(error_) as never
    if (code === ChainDisconnectedError.code)
      return new ChainDisconnectedError(error_) as never
    if (code === UserRejectedRequestError.code)
      return new UserRejectedRequestError(error_) as never
    if (code === UnauthorizedError.code)
      return new UnauthorizedError(error_) as never
    if (code === UnsupportedMethodError.code)
      return new UnsupportedMethodError(error_) as never
    if (code === SwitchChainError.code)
      return new SwitchChainError(error_) as never
    if (code === AtomicReadyWalletRejectedUpgradeError.code)
      return new AtomicReadyWalletRejectedUpgradeError(error_) as never
    if (code === AtomicityNotSupportedError.code)
      return new AtomicityNotSupportedError(error_) as never
    if (code === BundleTooLargeError.code)
      return new BundleTooLargeError(error_) as never
    if (code === UnknownBundleIdError.code)
      return new UnknownBundleIdError(error_) as never
    if (code === DuplicateIdError.code)
      return new DuplicateIdError(error_) as never
    if (code === UnsupportedChainIdError.code)
      return new UnsupportedChainIdError(error_) as never
    if (code === UnsupportedNonOptionalCapabilityError.code)
      return new UnsupportedNonOptionalCapabilityError(error_) as never
  }
  return error_ as never
}

export declare namespace parseError {
  type ReturnType<
    errorObject extends RpcResponse.ErrorObject | unknown,
    //
    error = errorObject extends RpcResponse.ErrorObject
      ?
          | (errorObject['code'] extends DisconnectedError['code']
              ? DisconnectedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? DisconnectedError
              : never)
          | (errorObject['code'] extends ChainDisconnectedError['code']
              ? ChainDisconnectedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? ChainDisconnectedError
              : never)
          | (errorObject['code'] extends UserRejectedRequestError['code']
              ? UserRejectedRequestError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? UserRejectedRequestError
              : never)
          | (errorObject['code'] extends UnauthorizedError['code']
              ? UnauthorizedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? UnauthorizedError
              : never)
          | (errorObject['code'] extends UnsupportedMethodError['code']
              ? UnsupportedMethodError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? UnsupportedMethodError
              : never)
          | (errorObject['code'] extends SwitchChainError['code']
              ? SwitchChainError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? SwitchChainError
              : never)
          | (errorObject['code'] extends AtomicReadyWalletRejectedUpgradeError['code']
              ? AtomicReadyWalletRejectedUpgradeError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? AtomicReadyWalletRejectedUpgradeError
              : never)
          | (errorObject['code'] extends AtomicityNotSupportedError['code']
              ? AtomicityNotSupportedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? AtomicityNotSupportedError
              : never)
          | (errorObject['code'] extends BundleTooLargeError['code']
              ? BundleTooLargeError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? BundleTooLargeError
              : never)
          | (errorObject['code'] extends UnknownBundleIdError['code']
              ? UnknownBundleIdError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? UnknownBundleIdError
              : never)
          | (errorObject['code'] extends DuplicateIdError['code']
              ? DuplicateIdError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? DuplicateIdError
              : never)
          | (errorObject['code'] extends UnsupportedChainIdError['code']
              ? UnsupportedChainIdError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? UnsupportedChainIdError
              : never)
          | (errorObject['code'] extends UnsupportedNonOptionalCapabilityError['code']
              ? UnsupportedNonOptionalCapabilityError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? UnsupportedNonOptionalCapabilityError
              : never)
      : RpcResponse.parseError.ReturnType<RpcResponse.ErrorObject>,
  > = IsNever<error> extends true
    ? RpcResponse.parseError.ReturnType<errorObject>
    : error
}

/** Thrown when the provider is undefined. */
export class IsUndefinedError extends Errors.BaseError {
  override readonly name = 'Provider.IsUndefinedError'

  constructor() {
    super('`provider` is undefined.')
  }
}
