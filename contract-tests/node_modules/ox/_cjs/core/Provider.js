"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.IsUndefinedError = exports.ChainDisconnectedError = exports.DisconnectedError = exports.UnsupportedMethodError = exports.UnauthorizedError = exports.UserRejectedRequestError = exports.ProviderRpcError = void 0;
exports.createEmitter = createEmitter;
exports.from = from;
exports.parseError = parseError;
const eventemitter3_1 = require("eventemitter3");
const Errors = require("./Errors.js");
const RpcResponse = require("./RpcResponse.js");
class ProviderRpcError extends Error {
    constructor(code, message) {
        super(message);
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'ProviderRpcError'
        });
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        Object.defineProperty(this, "details", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        this.code = code;
        this.details = message;
    }
}
exports.ProviderRpcError = ProviderRpcError;
class UserRejectedRequestError extends ProviderRpcError {
    constructor({ message = 'The user rejected the request.', } = {}) {
        super(4001, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 4001
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.UserRejectedRequestError'
        });
    }
}
exports.UserRejectedRequestError = UserRejectedRequestError;
Object.defineProperty(UserRejectedRequestError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4001
});
class UnauthorizedError extends ProviderRpcError {
    constructor({ message = 'The requested method and/or account has not been authorized by the user.', } = {}) {
        super(4100, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 4100
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.UnauthorizedError'
        });
    }
}
exports.UnauthorizedError = UnauthorizedError;
Object.defineProperty(UnauthorizedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4100
});
class UnsupportedMethodError extends ProviderRpcError {
    constructor({ message = 'The provider does not support the requested method.', } = {}) {
        super(4200, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 4200
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.UnsupportedMethodError'
        });
    }
}
exports.UnsupportedMethodError = UnsupportedMethodError;
Object.defineProperty(UnsupportedMethodError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4200
});
class DisconnectedError extends ProviderRpcError {
    constructor({ message = 'The provider is disconnected from all chains.', } = {}) {
        super(4900, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 4900
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.DisconnectedError'
        });
    }
}
exports.DisconnectedError = DisconnectedError;
Object.defineProperty(DisconnectedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4900
});
class ChainDisconnectedError extends ProviderRpcError {
    constructor({ message = 'The provider is not connected to the requested chain.', } = {}) {
        super(4901, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 4901
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.ChainDisconnectedError'
        });
    }
}
exports.ChainDisconnectedError = ChainDisconnectedError;
Object.defineProperty(ChainDisconnectedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4901
});
function createEmitter() {
    const emitter = new eventemitter3_1.EventEmitter();
    return {
        get eventNames() {
            return emitter.eventNames.bind(emitter);
        },
        get listenerCount() {
            return emitter.listenerCount.bind(emitter);
        },
        get listeners() {
            return emitter.listeners.bind(emitter);
        },
        addListener: emitter.addListener.bind(emitter),
        emit: emitter.emit.bind(emitter),
        off: emitter.off.bind(emitter),
        on: emitter.on.bind(emitter),
        once: emitter.once.bind(emitter),
        removeAllListeners: emitter.removeAllListeners.bind(emitter),
        removeListener: emitter.removeListener.bind(emitter),
    };
}
function from(provider, options = {}) {
    const { includeEvents = true } = options;
    if (!provider)
        throw new IsUndefinedError();
    return {
        ...(includeEvents
            ? {
                on: provider.on?.bind(provider),
                removeListener: provider.removeListener?.bind(provider),
            }
            : {}),
        async request(args) {
            try {
                const result = await provider.request(args);
                if (result &&
                    typeof result === 'object' &&
                    'jsonrpc' in result)
                    return RpcResponse.parse(result);
                return result;
            }
            catch (error) {
                throw parseError(error);
            }
        },
    };
}
function parseError(errorObject) {
    const errorObject_ = errorObject;
    const error = RpcResponse.parseError(errorObject_);
    if (error instanceof RpcResponse.InternalError) {
        if (!error.data)
            return error;
        const { code } = error.data;
        if (code === DisconnectedError.code)
            return new DisconnectedError(errorObject_);
        if (code === ChainDisconnectedError.code)
            return new ChainDisconnectedError(errorObject_);
        if (code === UserRejectedRequestError.code)
            return new UserRejectedRequestError(errorObject_);
        if (code === UnauthorizedError.code)
            return new UnauthorizedError(errorObject_);
        if (code === UnsupportedMethodError.code)
            return new UnsupportedMethodError(errorObject_);
    }
    return error;
}
class IsUndefinedError extends Errors.BaseError {
    constructor() {
        super('`provider` is undefined.');
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.IsUndefinedError'
        });
    }
}
exports.IsUndefinedError = IsUndefinedError;
//# sourceMappingURL=Provider.js.map