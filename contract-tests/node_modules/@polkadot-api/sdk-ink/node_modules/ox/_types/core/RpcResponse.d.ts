import type { Errors, RpcRequest } from '../index.js';
import type { Compute, IsNarrowable, IsNever, OneOf, UnionPartialBy } from './internal/types.js';
/** A JSON-RPC response object as per the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification#request_object). */
export type RpcResponse<result = unknown, error extends ErrorObject = ErrorObject> = Compute<{
    id: number;
    jsonrpc: '2.0';
} & OneOf<{
    result: result;
} | {
    error: error;
}>>;
/** JSON-RPC error object as per the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification#error_object). */
export type ErrorObject = {
    code: number;
    message: string;
    data?: unknown | undefined;
};
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
export declare function from<request extends RpcRequest.RpcRequest | undefined = undefined, const response = (request extends RpcRequest.RpcRequest ? request['_returnType'] : RpcResponse) | unknown>(response: from.Response<request, response>, options?: from.Options<request>): Compute<from.ReturnType<response>>;
export declare namespace from {
    type Response<request extends RpcRequest.RpcRequest | undefined = undefined, response = unknown> = response & (request extends RpcRequest.RpcRequest ? UnionPartialBy<RpcResponse<request['_returnType']>, 'id' | 'jsonrpc'> : RpcResponse);
    type Options<request extends RpcRequest.RpcRequest | undefined = RpcRequest.RpcRequest | undefined> = {
        request?: request | RpcRequest.RpcRequest | undefined;
    };
    type ReturnType<response> = IsNarrowable<response, RpcResponse> extends true ? RpcResponse : response & Readonly<{
        id: number;
        jsonrpc: '2.0';
    }>;
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
export declare function parse<const response extends RpcResponse | unknown, returnType, raw extends boolean = false>(response: response, options?: parse.Options<returnType, raw>): parse.ReturnType<unknown extends response ? returnType : response extends RpcResponse ? response extends {
    result: infer result;
} ? result : never : returnType, raw>;
export declare namespace parse {
    type Options<returnType, raw extends boolean = false> = {
        /**
         * JSON-RPC Method that was used to make the request. Used for typing the response.
         */
        request?: {
            _returnType: returnType;
        } | RpcRequest.RpcRequest | undefined;
        /**
         * Enables raw mode – responses will return an object with `result` and `error` properties instead of returning the `result` directly and throwing errors.
         *
         * - `true`: a JSON-RPC response object will be returned with `result` and `error` properties.
         * - `false`: the JSON-RPC response object's `result` property will be returned directly, and JSON-RPC Errors will be thrown.
         *
         * @default false
         */
        raw?: raw | boolean | undefined;
    };
    type ReturnType<returnType, raw extends boolean = false> = Compute<raw extends true ? RpcResponse<returnType> : returnType>;
    type ErrorType = ParseError | InvalidInputError | ResourceNotFoundError | ResourceUnavailableError | TransactionRejectedError | MethodNotSupportedError | LimitExceededError | VersionNotSupportedError | InvalidRequestError | MethodNotFoundError | InvalidParamsError | InternalError | BaseErrorType | Errors.GlobalErrorType;
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
export declare function parseError<const error extends Error | ErrorObject | unknown>(error: error | Error | ErrorObject): parseError.ReturnType<error>;
export declare namespace parseError {
    type ReturnType<errorObject extends ErrorObject | unknown, error = errorObject extends ErrorObject ? (errorObject['code'] extends InternalError['code'] ? InternalError : never) | (IsNarrowable<errorObject['code'], number> extends false ? InternalError : never) | (errorObject['code'] extends InvalidInputError['code'] ? InvalidInputError : never) | (IsNarrowable<errorObject['code'], number> extends false ? InvalidInputError : never) | (errorObject['code'] extends ResourceNotFoundError['code'] ? ResourceNotFoundError : never) | (IsNarrowable<errorObject['code'], number> extends false ? ResourceNotFoundError : never) | (errorObject['code'] extends ResourceUnavailableError['code'] ? ResourceUnavailableError : never) | (IsNarrowable<errorObject['code'], number> extends false ? ResourceUnavailableError : never) | (errorObject['code'] extends TransactionRejectedError['code'] ? TransactionRejectedError : never) | (IsNarrowable<errorObject['code'], number> extends false ? TransactionRejectedError : never) | (errorObject['code'] extends ParseError['code'] ? ParseError : never) | (IsNarrowable<errorObject['code'], number> extends false ? ParseError : never) | (errorObject['code'] extends MethodNotSupportedError['code'] ? MethodNotSupportedError : never) | (IsNarrowable<errorObject['code'], number> extends false ? MethodNotSupportedError : never) | (errorObject['code'] extends LimitExceededError['code'] ? LimitExceededError : never) | (IsNarrowable<errorObject['code'], number> extends false ? LimitExceededError : never) | (errorObject['code'] extends VersionNotSupportedError['code'] ? VersionNotSupportedError : never) | (IsNarrowable<errorObject['code'], number> extends false ? VersionNotSupportedError : never) | (errorObject['code'] extends InvalidRequestError['code'] ? InvalidRequestError : never) | (IsNarrowable<errorObject['code'], number> extends false ? InvalidRequestError : never) | (errorObject['code'] extends MethodNotFoundError['code'] ? MethodNotFoundError : never) | (IsNarrowable<errorObject['code'], number> extends false ? MethodNotFoundError : never) | (errorObject['code'] extends InvalidParamsError['code'] ? InvalidParamsError : never) | (IsNarrowable<errorObject['code'], number> extends false ? InvalidParamsError : never) | (IsNarrowable<errorObject['code'], number> extends false ? BaseError : never) : parseError.ReturnType<ErrorObject>> = IsNever<error> extends true ? BaseError : error;
}
export type BaseErrorType = BaseError & {
    name: 'BaseError';
};
/** Thrown when a JSON-RPC error has occurred. */
export declare class BaseError extends Error {
    name: string;
    readonly cause: Error | undefined;
    readonly stack: string;
    readonly code: number;
    readonly data?: unknown | undefined;
    constructor(errorObject: ErrorObject & {
        cause?: Error | undefined;
        stack?: string | undefined;
    });
}
/** Thrown when the input to a JSON-RPC method is invalid. */
export declare class InvalidInputError extends BaseError {
    static readonly code = -32000;
    readonly code = -32000;
    readonly name = "RpcResponse.InvalidInputError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC resource is not found. */
export declare class ResourceNotFoundError extends BaseError {
    static readonly code = -32001;
    readonly code = -32001;
    readonly name = "RpcResponse.ResourceNotFoundError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC resource is unavailable. */
export declare class ResourceUnavailableError extends BaseError {
    static readonly code = -32002;
    readonly code = -32002;
    readonly name = "RpcResponse.ResourceUnavailableError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC transaction is rejected. */
export declare class TransactionRejectedError extends BaseError {
    static readonly code = -32003;
    readonly code = -32003;
    readonly name = "RpcResponse.TransactionRejectedError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC method is not supported. */
export declare class MethodNotSupportedError extends BaseError {
    static readonly code = -32004;
    readonly code = -32004;
    readonly name = "RpcResponse.MethodNotSupportedError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a rate-limit is exceeded. */
export declare class LimitExceededError extends BaseError {
    static readonly code = -32005;
    readonly code = -32005;
    readonly name = "RpcResponse.LimitExceededError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC version is not supported. */
export declare class VersionNotSupportedError extends BaseError {
    static readonly code = -32006;
    readonly code = -32006;
    readonly name = "RpcResponse.VersionNotSupportedError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC request is invalid. */
export declare class InvalidRequestError extends BaseError {
    static readonly code = -32600;
    readonly code = -32600;
    readonly name = "RpcResponse.InvalidRequestError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when a JSON-RPC method is not found. */
export declare class MethodNotFoundError extends BaseError {
    static readonly code = -32601;
    readonly code = -32601;
    readonly name = "RpcResponse.MethodNotFoundError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when the parameters to a JSON-RPC method are invalid. */
export declare class InvalidParamsError extends BaseError {
    static readonly code = -32602;
    readonly code = -32602;
    readonly name = "RpcResponse.InvalidParamsError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
/** Thrown when an internal JSON-RPC error has occurred. */
export declare class InternalError extends BaseError {
    static readonly code = -32603;
    readonly code = -32603;
    readonly name = "RpcResponse.InternalError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>> & {
        cause?: Error | undefined;
        stack?: string | undefined;
    });
}
/** Thrown when a JSON-RPC response is invalid. */
export declare class ParseError extends BaseError {
    static readonly code = -32700;
    readonly code = -32700;
    readonly name = "RpcResponse.ParseError";
    constructor(parameters?: Partial<Omit<ErrorObject, 'code'>>);
}
//# sourceMappingURL=RpcResponse.d.ts.map