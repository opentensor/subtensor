import type { Errors, RpcRequest } from '../index.js'
import type {
  Compute,
  IsNarrowable,
  IsNever,
  OneOf,
  UnionPartialBy,
} from './internal/types.js'

/** A JSON-RPC response object as per the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification#request_object). */
export type RpcResponse<
  result = unknown,
  error extends ErrorObject = ErrorObject,
> = Compute<
  {
    id: number
    jsonrpc: '2.0'
  } & OneOf<{ result: result } | { error: error }>
>

/** JSON-RPC error object as per the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification#error_object). */
export type ErrorObject = {
  code: number
  message: string
  data?: unknown | undefined
}

/**
 * A type-safe interface to instantiate a JSON-RPC response object as per the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification#response_object).
 *
 * @example
 * ### Instantiating a Response Object
 *
 * ```ts twoslash
 * import { RpcResponse } from 'ox'
 *
 * const response = RpcResponse.from({
 *   id: 0,
 *   jsonrpc: '2.0',
 *   result: '0x69420',
 * })
 * ```
 *
 * @example
 * ### Type-safe Instantiation
 *
 * If you have a JSON-RPC request object, you can use it to strongly-type the response. If a `request` is provided,
 * then the `id` and `jsonrpc` properties will be overridden with the values from the request.
 *
 * ```ts twoslash
 * import { RpcRequest, RpcResponse } from 'ox'
 *
 * const request = RpcRequest.from({ id: 0, method: 'eth_blockNumber' })
 *
 * const response = RpcResponse.from(
 *   { result: '0x69420' },
 *   { request },
 * )
 * ```
 *
 * @param response - Opaque JSON-RPC response object.
 * @param options - Parsing options.
 * @returns Typed JSON-RPC result, or response object (if `raw` is `true`).
 */
export function from<
  request extends RpcRequest.RpcRequest | undefined = undefined,
  const response =
    | (request extends RpcRequest.RpcRequest
        ? request['_returnType']
        : RpcResponse)
    | unknown,
>(
  response: from.Response<request, response>,
  options?: from.Options<request>,
): Compute<from.ReturnType<response>>
// eslint-disable-next-line jsdoc/require-jsdoc
export function from(response: RpcResponse, options: any = {}): RpcResponse {
  const { request } = options
  return {
    ...response,
    id: response.id ?? request?.id,
    jsonrpc: response.jsonrpc ?? request.jsonrpc,
  }
}

export declare namespace from {
  type Response<
    request extends RpcRequest.RpcRequest | undefined = undefined,
    response = unknown,
  > = response &
    (request extends RpcRequest.RpcRequest
      ? UnionPartialBy<RpcResponse<request['_returnType']>, 'id' | 'jsonrpc'>
      : RpcResponse)

  type Options<
    request extends RpcRequest.RpcRequest | undefined =
      | RpcRequest.RpcRequest
      | undefined,
  > = {
    request?: request | RpcRequest.RpcRequest | undefined
  }

  type ReturnType<response> = IsNarrowable<response, RpcResponse> extends true
    ? RpcResponse
    : response & Readonly<{ id: number; jsonrpc: '2.0' }>
}

/**
 * A type-safe interface to parse a JSON-RPC response object as per the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification#response_object), and extract the result.
 *
 * @example
 * ```ts twoslash
 * import { RpcRequest, RpcResponse } from 'ox'
 *
 * // 1. Create a request store.
 * const store = RpcRequest.createStore()
 *
 * // 2. Get a request object.
 * const request = store.prepare({
 *   method: 'eth_getBlockByNumber',
 *   params: ['0x1', false],
 * })
 *
 * // 3. Send the JSON-RPC request via HTTP.
 * const block = await fetch('https://1.rpc.thirdweb.com', {
 *   body: JSON.stringify(request),
 *   headers: {
 *     'Content-Type': 'application/json',
 *   },
 *   method: 'POST',
 * })
 *  .then((response) => response.json())
 *  // 4. Parse the JSON-RPC response into a type-safe result. // [!code focus]
 *  .then((response) => RpcResponse.parse(response, { request })) // [!code focus]
 *
 * block // [!code focus]
 * // ^?
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
 *
 * :::tip
 *
 * If you don't need the return type, you can omit the options entirely.
 *
 * ```ts twoslash
 * // @noErrors
 * import { RpcResponse } from 'ox'
 *
 * const block = await fetch('https://1.rpc.thirdweb.com', {})
 *  .then((response) => response.json())
 *  .then((response) => RpcResponse.parse(response, { request })) // [!code --]
 *  .then(RpcResponse.parse) // [!code ++]
 * ```
 * :::
 *
 * @example
 * ### Raw Mode
 *
 * If `raw` is `true`, the response will be returned as an object with `result` and `error` properties instead of returning the `result` directly and throwing errors.
 *
 * ```ts twoslash
 * import { RpcRequest, RpcResponse } from 'ox'
 *
 * const store = RpcRequest.createStore()
 *
 * const request = store.prepare({
 *   method: 'eth_blockNumber',
 * })
 *
 * const response = RpcResponse.parse({}, {
 *   request,
 *   raw: true, // [!code hl]
 * })
 *
 * response.result
 * //       ^?
 *
 *
 * response.error
 * //       ^?
 *
 *
 * ```
 *
 * @param response - Opaque JSON-RPC response object.
 * @param options - Parsing options.
 * @returns Typed JSON-RPC result, or response object (if `raw` is `true`).
 */
export function parse<
  const response extends RpcResponse | unknown,
  returnType,
  raw extends boolean = false,
>(
  response: response,
  options: parse.Options<returnType, raw> = {},
): parse.ReturnType<
  unknown extends response
    ? returnType
    : response extends RpcResponse
      ? response extends { result: infer result }
        ? result
        : never
      : returnType,
  raw
> {
  const { raw = false } = options
  const response_ = response as RpcResponse
  if (raw) return response as never
  if (response_.error) throw parseError(response_.error)
  return response_.result as never
}

export declare namespace parse {
  type Options<returnType, raw extends boolean = false> = {
    /**
     * JSON-RPC Method that was used to make the request. Used for typing the response.
     */
    request?:
      | {
          _returnType: returnType
        }
      | RpcRequest.RpcRequest
      | undefined
    /**
     * Enables raw mode – responses will return an object with `result` and `error` properties instead of returning the `result` directly and throwing errors.
     *
     * - `true`: a JSON-RPC response object will be returned with `result` and `error` properties.
     * - `false`: the JSON-RPC response object's `result` property will be returned directly, and JSON-RPC Errors will be thrown.
     *
     * @default false
     */
    raw?: raw | boolean | undefined
  }

  type ReturnType<returnType, raw extends boolean = false> = Compute<
    raw extends true ? RpcResponse<returnType> : returnType
  >

  type ErrorType =
    | ParseError
    | InvalidInputError
    | ResourceNotFoundError
    | ResourceUnavailableError
    | TransactionRejectedError
    | MethodNotSupportedError
    | LimitExceededError
    | VersionNotSupportedError
    | InvalidRequestError
    | MethodNotFoundError
    | InvalidParamsError
    | InternalError
    | BaseErrorType
    | Errors.GlobalErrorType
}

/**
 * Parses an error into a RPC Error instance.
 *
 * @example
 * ```ts twoslash
 * import { RpcResponse } from 'ox'
 *
 * const error = RpcResponse.parseError({ code: -32000, message: 'unsupported method' })
 *
 * error
 * // ^?
 *
 * ```
 *
 * @param error - Error.
 * @returns RPC Error instance.
 */
export function parseError<const error extends Error | ErrorObject | unknown>(
  error: error | Error | ErrorObject,
): parseError.ReturnType<error> {
  const error_ = error as Error | ErrorObject

  if (error_ instanceof Error && !('code' in error_))
    return new InternalError({
      cause: error_,
      data: error_,
      message: error_.message,
      stack: error_.stack,
    }) as never

  const { code } = error_
  if (code === InternalError.code)
    return new InternalError(error_ as never) as never
  if (code === InvalidInputError.code)
    return new InvalidInputError(error_) as never
  if (code === InvalidParamsError.code)
    return new InvalidParamsError(error_) as never
  if (code === InvalidRequestError.code)
    return new InvalidRequestError(error_) as never
  if (code === LimitExceededError.code)
    return new LimitExceededError(error_) as never
  if (code === MethodNotFoundError.code)
    return new MethodNotFoundError(error_) as never
  if (code === MethodNotSupportedError.code)
    return new MethodNotSupportedError(error_) as never
  if (code === ParseError.code) return new ParseError(error_) as never
  if (code === ResourceNotFoundError.code)
    return new ResourceNotFoundError(error_) as never
  if (code === ResourceUnavailableError.code)
    return new ResourceUnavailableError(error_) as never
  if (code === TransactionRejectedError.code)
    return new TransactionRejectedError(error_) as never
  if (code === VersionNotSupportedError.code)
    return new VersionNotSupportedError(error_) as never
  return new InternalError({
    cause: error_ instanceof Error ? error_ : undefined,
    data: error_,
    message: error_.message,
    stack: error_ instanceof Error ? error_.stack : undefined,
  }) as never
}

export declare namespace parseError {
  type ReturnType<
    errorObject extends ErrorObject | unknown,
    //
    error = errorObject extends ErrorObject
      ?
          | (errorObject['code'] extends InternalError['code']
              ? InternalError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? InternalError
              : never)
          | (errorObject['code'] extends InvalidInputError['code']
              ? InvalidInputError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? InvalidInputError
              : never)
          | (errorObject['code'] extends ResourceNotFoundError['code']
              ? ResourceNotFoundError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? ResourceNotFoundError
              : never)
          | (errorObject['code'] extends ResourceUnavailableError['code']
              ? ResourceUnavailableError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? ResourceUnavailableError
              : never)
          | (errorObject['code'] extends TransactionRejectedError['code']
              ? TransactionRejectedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? TransactionRejectedError
              : never)
          | (errorObject['code'] extends ParseError['code']
              ? ParseError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? ParseError
              : never)
          | (errorObject['code'] extends MethodNotSupportedError['code']
              ? MethodNotSupportedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? MethodNotSupportedError
              : never)
          | (errorObject['code'] extends LimitExceededError['code']
              ? LimitExceededError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? LimitExceededError
              : never)
          | (errorObject['code'] extends VersionNotSupportedError['code']
              ? VersionNotSupportedError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? VersionNotSupportedError
              : never)
          | (errorObject['code'] extends InvalidRequestError['code']
              ? InvalidRequestError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? InvalidRequestError
              : never)
          | (errorObject['code'] extends MethodNotFoundError['code']
              ? MethodNotFoundError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? MethodNotFoundError
              : never)
          | (errorObject['code'] extends InvalidParamsError['code']
              ? InvalidParamsError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? InvalidParamsError
              : never)
          | (IsNarrowable<errorObject['code'], number> extends false
              ? BaseError
              : never)
      : parseError.ReturnType<ErrorObject>,
  > = IsNever<error> extends true ? BaseError : error
}

export type BaseErrorType = BaseError & { name: 'BaseError' }

/** Thrown when a JSON-RPC error has occurred. */
export class BaseError extends Error {
  override name = 'RpcResponse.BaseError'

  override readonly cause: Error | undefined
  override readonly stack: string
  readonly code: number
  readonly data?: unknown | undefined

  constructor(
    errorObject: ErrorObject & {
      cause?: Error | undefined
      stack?: string | undefined
    },
  ) {
    const { cause, code, message, data, stack } = errorObject

    super(message, { cause })

    this.cause = cause
    this.code = code
    this.data = data
    this.stack = stack ?? ''
  }
}

/** Thrown when the input to a JSON-RPC method is invalid. */
export class InvalidInputError extends BaseError {
  static readonly code = -32000
  override readonly code = -32000
  override readonly name = 'RpcResponse.InvalidInputError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: InvalidInputError.code,
      data: parameters.data,
      message: parameters.message ?? 'Missing or invalid parameters.',
    })
  }
}

/** Thrown when a JSON-RPC resource is not found. */
export class ResourceNotFoundError extends BaseError {
  static readonly code = -32001
  override readonly code = -32001
  override readonly name = 'RpcResponse.ResourceNotFoundError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: ResourceNotFoundError.code,
      data: parameters.data,
      message: parameters.message ?? 'Requested resource not found.',
    })
  }
}

/** Thrown when a JSON-RPC resource is unavailable. */
export class ResourceUnavailableError extends BaseError {
  static readonly code = -32002
  override readonly code = -32002
  override readonly name = 'RpcResponse.ResourceUnavailableError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: ResourceUnavailableError.code,
      data: parameters.data,
      message: parameters.message ?? 'Requested resource not available.',
    })
  }
}

/** Thrown when a JSON-RPC transaction is rejected. */
export class TransactionRejectedError extends BaseError {
  static readonly code = -32003
  override readonly code = -32003
  override readonly name = 'RpcResponse.TransactionRejectedError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: TransactionRejectedError.code,
      data: parameters.data,
      message: parameters.message ?? 'Transaction creation failed.',
    })
  }
}

/** Thrown when a JSON-RPC method is not supported. */
export class MethodNotSupportedError extends BaseError {
  static readonly code = -32004
  override readonly code = -32004
  override readonly name = 'RpcResponse.MethodNotSupportedError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: MethodNotSupportedError.code,
      data: parameters.data,
      message: parameters.message ?? 'Method is not implemented.',
    })
  }
}

/** Thrown when a rate-limit is exceeded. */
export class LimitExceededError extends BaseError {
  static readonly code = -32005
  override readonly code = -32005
  override readonly name = 'RpcResponse.LimitExceededError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: LimitExceededError.code,
      data: parameters.data,
      message: parameters.message ?? 'Rate limit exceeded.',
    })
  }
}

/** Thrown when a JSON-RPC version is not supported. */
export class VersionNotSupportedError extends BaseError {
  static readonly code = -32006
  override readonly code = -32006
  override readonly name = 'RpcResponse.VersionNotSupportedError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: VersionNotSupportedError.code,
      data: parameters.data,
      message: parameters.message ?? 'JSON-RPC version not supported.',
    })
  }
}

/** Thrown when a JSON-RPC request is invalid. */
export class InvalidRequestError extends BaseError {
  static readonly code = -32600
  override readonly code = -32600
  override readonly name = 'RpcResponse.InvalidRequestError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: InvalidRequestError.code,
      data: parameters.data,
      message: parameters.message ?? 'Input is not a valid JSON-RPC request.',
    })
  }
}

/** Thrown when a JSON-RPC method is not found. */
export class MethodNotFoundError extends BaseError {
  static readonly code = -32601
  override readonly code = -32601
  override readonly name = 'RpcResponse.MethodNotFoundError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: MethodNotFoundError.code,
      data: parameters.data,
      message: parameters.message ?? 'Method does not exist.',
    })
  }
}

/** Thrown when the parameters to a JSON-RPC method are invalid. */
export class InvalidParamsError extends BaseError {
  static readonly code = -32602
  override readonly code = -32602
  override readonly name = 'RpcResponse.InvalidParamsError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: InvalidParamsError.code,
      data: parameters.data,
      message: parameters.message ?? 'Invalid method parameters.',
    })
  }
}

/** Thrown when an internal JSON-RPC error has occurred. */
export class InternalError extends BaseError {
  static readonly code = -32603
  override readonly code = -32603
  override readonly name = 'RpcResponse.InternalError'

  constructor(
    parameters: Partial<Omit<ErrorObject, 'code'>> & {
      cause?: Error | undefined
      stack?: string | undefined
    } = {},
  ) {
    super({
      cause: parameters.cause,
      code: InternalError.code,
      data: parameters.data,
      message: parameters.message ?? 'Internal JSON-RPC error.',
      stack: parameters.stack,
    })
  }
}

/** Thrown when a JSON-RPC response is invalid. */
export class ParseError extends BaseError {
  static readonly code = -32700
  override readonly code = -32700
  override readonly name = 'RpcResponse.ParseError'

  constructor(parameters: Partial<Omit<ErrorObject, 'code'>> = {}) {
    super({
      code: ParseError.code,
      data: parameters.data,
      message: parameters.message ?? 'Failed to parse JSON-RPC response.',
    })
  }
}
