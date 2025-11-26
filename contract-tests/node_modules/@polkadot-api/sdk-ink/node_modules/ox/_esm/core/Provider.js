import { EventEmitter } from 'eventemitter3';
import * as Errors from './Errors.js';
import * as RpcResponse from './RpcResponse.js';
export class ProviderRpcError extends Error {
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
/** The user rejected the request. */
export class UserRejectedRequestError extends ProviderRpcError {
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
Object.defineProperty(UserRejectedRequestError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4001
});
/** The requested method and/or account has not been authorized by the user. */
export class UnauthorizedError extends ProviderRpcError {
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
Object.defineProperty(UnauthorizedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4100
});
/** The provider does not support the requested method. */
export class UnsupportedMethodError extends ProviderRpcError {
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
Object.defineProperty(UnsupportedMethodError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4200
});
/** The provider is disconnected from all chains. */
export class DisconnectedError extends ProviderRpcError {
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
Object.defineProperty(DisconnectedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4900
});
/** The provider is not connected to the requested chain. */
export class ChainDisconnectedError extends ProviderRpcError {
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
Object.defineProperty(ChainDisconnectedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4901
});
/** An error occurred when attempting to switch chain. */
export class SwitchChainError extends ProviderRpcError {
    constructor({ message = 'An error occurred when attempting to switch chain.', } = {}) {
        super(4902, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 4902
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.SwitchChainError'
        });
    }
}
Object.defineProperty(SwitchChainError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 4902
});
/** This Wallet does not support a capability that was not marked as optional. */
export class UnsupportedNonOptionalCapabilityError extends ProviderRpcError {
    constructor({ message = 'This Wallet does not support a capability that was not marked as optional.', } = {}) {
        super(5700, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5700
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.UnsupportedNonOptionalCapabilityError'
        });
    }
}
Object.defineProperty(UnsupportedNonOptionalCapabilityError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5700
});
/** This Wallet does not support the requested chain ID. */
export class UnsupportedChainIdError extends ProviderRpcError {
    constructor({ message = 'This Wallet does not support the requested chain ID.', } = {}) {
        super(5710, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5710
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.UnsupportedChainIdError'
        });
    }
}
Object.defineProperty(UnsupportedChainIdError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5710
});
/** There is already a bundle submitted with this ID. */
export class DuplicateIdError extends ProviderRpcError {
    constructor({ message = 'There is already a bundle submitted with this ID.', } = {}) {
        super(5720, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5720
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.DuplicateIdError'
        });
    }
}
Object.defineProperty(DuplicateIdError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5720
});
/** This bundle id is unknown / has not been submitted. */
export class UnknownBundleIdError extends ProviderRpcError {
    constructor({ message = 'This bundle id is unknown / has not been submitted.', } = {}) {
        super(5730, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5730
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.UnknownBundleIdError'
        });
    }
}
Object.defineProperty(UnknownBundleIdError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5730
});
/** The call bundle is too large for the Wallet to process. */
export class BundleTooLargeError extends ProviderRpcError {
    constructor({ message = 'The call bundle is too large for the Wallet to process.', } = {}) {
        super(5740, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5740
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.BundleTooLargeError'
        });
    }
}
Object.defineProperty(BundleTooLargeError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5740
});
/** The Wallet can support atomicity after an upgrade, but the user rejected the upgrade. */
export class AtomicReadyWalletRejectedUpgradeError extends ProviderRpcError {
    constructor({ message = 'The Wallet can support atomicity after an upgrade, but the user rejected the upgrade.', } = {}) {
        super(5750, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5750
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.AtomicReadyWalletRejectedUpgradeError'
        });
    }
}
Object.defineProperty(AtomicReadyWalletRejectedUpgradeError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5750
});
/** The wallet does not support atomic execution but the request requires it. */
export class AtomicityNotSupportedError extends ProviderRpcError {
    constructor({ message = 'The wallet does not support atomic execution but the request requires it.', } = {}) {
        super(5760, message);
        Object.defineProperty(this, "code", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 5760
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'Provider.AtomicityNotSupportedError'
        });
    }
}
Object.defineProperty(AtomicityNotSupportedError, "code", {
    enumerable: true,
    configurable: true,
    writable: true,
    value: 5760
});
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
export function createEmitter() {
    const emitter = new EventEmitter();
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
// eslint-disable-next-line jsdoc/require-jsdoc
export function from(provider, options = {}) {
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
export function parseError(error) {
    const error_ = RpcResponse.parseError(error);
    if (error_ instanceof RpcResponse.InternalError) {
        if (!error_.data)
            return error_;
        const { code } = error_.data;
        if (code === DisconnectedError.code)
            return new DisconnectedError(error_);
        if (code === ChainDisconnectedError.code)
            return new ChainDisconnectedError(error_);
        if (code === UserRejectedRequestError.code)
            return new UserRejectedRequestError(error_);
        if (code === UnauthorizedError.code)
            return new UnauthorizedError(error_);
        if (code === UnsupportedMethodError.code)
            return new UnsupportedMethodError(error_);
        if (code === SwitchChainError.code)
            return new SwitchChainError(error_);
        if (code === AtomicReadyWalletRejectedUpgradeError.code)
            return new AtomicReadyWalletRejectedUpgradeError(error_);
        if (code === AtomicityNotSupportedError.code)
            return new AtomicityNotSupportedError(error_);
        if (code === BundleTooLargeError.code)
            return new BundleTooLargeError(error_);
        if (code === UnknownBundleIdError.code)
            return new UnknownBundleIdError(error_);
        if (code === DuplicateIdError.code)
            return new DuplicateIdError(error_);
        if (code === UnsupportedChainIdError.code)
            return new UnsupportedChainIdError(error_);
        if (code === UnsupportedNonOptionalCapabilityError.code)
            return new UnsupportedNonOptionalCapabilityError(error_);
    }
    return error_;
}
/** Thrown when the provider is undefined. */
export class IsUndefinedError extends Errors.BaseError {
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
//# sourceMappingURL=Provider.js.map