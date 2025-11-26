"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ParseError = exports.InternalError = exports.InvalidParamsError = exports.MethodNotFoundError = exports.InvalidRequestError = exports.VersionNotSupportedError = exports.LimitExceededError = exports.MethodNotSupportedError = exports.TransactionRejectedError = exports.ResourceUnavailableError = exports.ResourceNotFoundError = exports.InvalidInputError = exports.BaseError = void 0;
exports.from = from;
exports.parse = parse;
exports.parseError = parseError;
function from(response, options = {}) {
    const { request } = options;
    return {
        ...response,
        id: response.id ?? request?.id,
        jsonrpc: response.jsonrpc ?? request.jsonrpc,
    };
}
function parse(response, options = {}) {
    const { raw = false } = options;
    const response_ = response;
    if (raw)
        return response;
    if (response_.error)
        throw parseError(response_.error);
    return response_.result;
}
function parseError(errorObject) {
    const errorObject_ = errorObject;
    const { code } = errorObject_;
    if (code === InternalError.code)
        return new InternalError(errorObject_);
    if (code === InvalidInputError.code)
        return new InvalidInputError(errorObject_);
    if (code === InvalidParamsError.code)
        return new InvalidParamsError(errorObject_);
    if (code === InvalidRequestError.code)
        return new InvalidRequestError(errorObject_);
    if (code === LimitExceededError.code)
        return new LimitExceededError(errorObject_);
    if (code === MethodNotFoundError.code)
        return new MethodNotFoundError(errorObject_);
    if (code === MethodNotSupportedError.code)
        return new MethodNotSupportedError(errorObject_);
    if (code === ParseError.code)
        return new ParseError(errorObject_);
    if (code === ResourceNotFoundError.code)
        return new ResourceNotFoundError(errorObject_);
    if (code === ResourceUnavailableError.code)
        return new ResourceUnavailableError(errorObject_);
    if (code === TransactionRejectedError.code)
        return new TransactionRejectedError(errorObject_);
    if (code === VersionNotSupportedError.code)
        return new VersionNotSupportedError(errorObject_);
    return new InternalError({
        data: errorObject_,
        message: errorObject_.message,
    });
}
class BaseError extends Error {
    constructor(errorObject) {
        const { code, message, data } = errorObject;
        super(message);
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'RpcResponse.BaseError'
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
        this.code = code;
        this.data = data;
    }
}
exports.BaseError = BaseError;
class InvalidInputError extends BaseError {
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
exports.InvalidInputError = InvalidInputError;
Object.defineProperty(InvalidInputError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32000
});
class ResourceNotFoundError extends BaseError {
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
exports.ResourceNotFoundError = ResourceNotFoundError;
Object.defineProperty(ResourceNotFoundError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32001
});
class ResourceUnavailableError extends BaseError {
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
exports.ResourceUnavailableError = ResourceUnavailableError;
Object.defineProperty(ResourceUnavailableError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32002
});
class TransactionRejectedError extends BaseError {
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
exports.TransactionRejectedError = TransactionRejectedError;
Object.defineProperty(TransactionRejectedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32003
});
class MethodNotSupportedError extends BaseError {
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
exports.MethodNotSupportedError = MethodNotSupportedError;
Object.defineProperty(MethodNotSupportedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32004
});
class LimitExceededError extends BaseError {
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
exports.LimitExceededError = LimitExceededError;
Object.defineProperty(LimitExceededError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32005
});
class VersionNotSupportedError extends BaseError {
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
exports.VersionNotSupportedError = VersionNotSupportedError;
Object.defineProperty(VersionNotSupportedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32006
});
class InvalidRequestError extends BaseError {
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
exports.InvalidRequestError = InvalidRequestError;
Object.defineProperty(InvalidRequestError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32600
});
class MethodNotFoundError extends BaseError {
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
exports.MethodNotFoundError = MethodNotFoundError;
Object.defineProperty(MethodNotFoundError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32601
});
class InvalidParamsError extends BaseError {
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
exports.InvalidParamsError = InvalidParamsError;
Object.defineProperty(InvalidParamsError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32602
});
class InternalError extends BaseError {
    constructor(parameters = {}) {
        super({
            code: InternalError.code,
            data: parameters.data,
            message: parameters.message ?? 'Internal JSON-RPC error.',
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
exports.InternalError = InternalError;
Object.defineProperty(InternalError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32603
});
class ParseError extends BaseError {
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
exports.ParseError = ParseError;
Object.defineProperty(ParseError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: -32700
});
//# sourceMappingURL=RpcResponse.js.map