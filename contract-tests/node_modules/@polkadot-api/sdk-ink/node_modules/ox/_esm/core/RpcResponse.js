// eslint-disable-next-line jsdoc/require-jsdoc
export function from(response, options = {}) {
    const { request } = options;
    return {
        ...response,
        id: response.id ?? request?.id,
        jsonrpc: response.jsonrpc ?? request.jsonrpc,
    };
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
export function parse(response, options = {}) {
    const { raw = false } = options;
    const response_ = response;
    if (raw)
        return response;
    if (response_.error)
        throw parseError(response_.error);
    return response_.result;
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
export function parseError(error) {
    const error_ = error;
    if (error_ instanceof Error && !('code' in error_))
        return new InternalError({
            cause: error_,
            data: error_,
            message: error_.message,
            stack: error_.stack,
        });
    const { code } = error_;
    if (code === InternalError.code)
        return new InternalError(error_);
    if (code === InvalidInputError.code)
        return new InvalidInputError(error_);
    if (code === InvalidParamsError.code)
        return new InvalidParamsError(error_);
    if (code === InvalidRequestError.code)
        return new InvalidRequestError(error_);
    if (code === LimitExceededError.code)
        return new LimitExceededError(error_);
    if (code === MethodNotFoundError.code)
        return new MethodNotFoundError(error_);
    if (code === MethodNotSupportedError.code)
        return new MethodNotSupportedError(error_);
    if (code === ParseError.code)
        return new ParseError(error_);
    if (code === ResourceNotFoundError.code)
        return new ResourceNotFoundError(error_);
    if (code === ResourceUnavailableError.code)
        return new ResourceUnavailableError(error_);
    if (code === TransactionRejectedError.code)
        return new TransactionRejectedError(error_);
    if (code === VersionNotSupportedError.code)
        return new VersionNotSupportedError(error_);
    return new InternalError({
        cause: error_ instanceof Error ? error_ : undefined,
        data: error_,
        message: error_.message,
        stack: error_ instanceof Error ? error_.stack : undefined,
    });
}
/** Thrown when a JSON-RPC error has occurred. */
export class BaseError extends Error {
    constructor(errorObject) {
        const { cause, code, message, data, stack } = errorObject;
        super(message, { cause });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.BaseError'
        });
        Object.defineProperty(this, "cause", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "stack", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "data", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        this.cause = cause;
        this.code = code;
        this.data = data;
        this.stack = stack ?? '';
    }
}
/** Thrown when the input to a JSON-RPC method is invalid. */
export class InvalidInputError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: InvalidInputError.code,
            data: parameters.data,
            message: parameters.message ?? 'Missing or invalid parameters.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32000
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.InvalidInputError'
        });
    }
}
Object.defineProperty(InvalidInputError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32000
});
/** Thrown when a JSON-RPC resource is not found. */
export class ResourceNotFoundError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: ResourceNotFoundError.code,
            data: parameters.data,
            message: parameters.message ?? 'Requested resource not found.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32001
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.ResourceNotFoundError'
        });
    }
}
Object.defineProperty(ResourceNotFoundError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32001
});
/** Thrown when a JSON-RPC resource is unavailable. */
export class ResourceUnavailableError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: ResourceUnavailableError.code,
            data: parameters.data,
            message: parameters.message ?? 'Requested resource not available.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32002
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.ResourceUnavailableError'
        });
    }
}
Object.defineProperty(ResourceUnavailableError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32002
});
/** Thrown when a JSON-RPC transaction is rejected. */
export class TransactionRejectedError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: TransactionRejectedError.code,
            data: parameters.data,
            message: parameters.message ?? 'Transaction creation failed.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32003
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.TransactionRejectedError'
        });
    }
}
Object.defineProperty(TransactionRejectedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32003
});
/** Thrown when a JSON-RPC method is not supported. */
export class MethodNotSupportedError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: MethodNotSupportedError.code,
            data: parameters.data,
            message: parameters.message ?? 'Method is not implemented.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32004
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.MethodNotSupportedError'
        });
    }
}
Object.defineProperty(MethodNotSupportedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32004
});
/** Thrown when a rate-limit is exceeded. */
export class LimitExceededError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: LimitExceededError.code,
            data: parameters.data,
            message: parameters.message ?? 'Rate limit exceeded.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32005
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.LimitExceededError'
        });
    }
}
Object.defineProperty(LimitExceededError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32005
});
/** Thrown when a JSON-RPC version is not supported. */
export class VersionNotSupportedError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: VersionNotSupportedError.code,
            data: parameters.data,
            message: parameters.message ?? 'JSON-RPC version not supported.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32006
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.VersionNotSupportedError'
        });
    }
}
Object.defineProperty(VersionNotSupportedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32006
});
/** Thrown when a JSON-RPC request is invalid. */
export class InvalidRequestError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: InvalidRequestError.code,
            data: parameters.data,
            message: parameters.message ?? 'Input is not a valid JSON-RPC request.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32600
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.InvalidRequestError'
        });
    }
}
Object.defineProperty(InvalidRequestError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32600
});
/** Thrown when a JSON-RPC method is not found. */
export class MethodNotFoundError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: MethodNotFoundError.code,
            data: parameters.data,
            message: parameters.message ?? 'Method does not exist.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32601
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.MethodNotFoundError'
        });
    }
}
Object.defineProperty(MethodNotFoundError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32601
});
/** Thrown when the parameters to a JSON-RPC method are invalid. */
export class InvalidParamsError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: InvalidParamsError.code,
            data: parameters.data,
            message: parameters.message ?? 'Invalid method parameters.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32602
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.InvalidParamsError'
        });
    }
}
Object.defineProperty(InvalidParamsError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32602
});
/** Thrown when an internal JSON-RPC error has occurred. */
export class InternalError extends BaseError {
    constructor(parameters = {}) {
        super({
            cause: parameters.cause,
            code: InternalError.code,
            data: parameters.data,
            message: parameters.message ?? 'Internal JSON-RPC error.',
            stack: parameters.stack,
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32603
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.InternalError'
        });
    }
}
Object.defineProperty(InternalError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32603
});
/** Thrown when a JSON-RPC response is invalid. */
export class ParseError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: ParseError.code,
            data: parameters.data,
            message: parameters.message ?? 'Failed to parse JSON-RPC response.',
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: -32700
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.ParseError'
        });
    }
}
Object.defineProperty(ParseError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32700
});
//# sourceMappingURL=RpcResponse.js.map