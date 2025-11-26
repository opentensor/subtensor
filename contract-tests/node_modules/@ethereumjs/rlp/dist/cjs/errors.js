"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.EthereumJSError = exports.DEFAULT_ERROR_CODE = void 0;
exports.EthereumJSErrorWithoutCode = EthereumJSErrorWithoutCode;
// In order to update all our errors to use `EthereumJSError`, temporarily include the
// unset error code. All errors throwing this code should be updated to use the relevant
// error code.
exports.DEFAULT_ERROR_CODE = 'ETHEREUMJS_DEFAULT_ERROR_CODE';
/**
 * Generic EthereumJS error with attached metadata
 */
class EthereumJSError extends Error {
    constructor(type, message, stack) {
        super(message ?? type.code);
        this.type = type;
        if (stack !== undefined)
            this.stack = stack;
    }
    getMetadata() {
        return this.type;
    }
    /**
     * Get the metadata and the stacktrace for the error.
     */
    toObject() {
        return {
            type: this.getMetadata(),
            message: this.message ?? '',
            stack: this.stack ?? '',
            className: this.constructor.name,
        };
    }
}
exports.EthereumJSError = EthereumJSError;
/**
 * @deprecated Use `EthereumJSError` with a set error code instead
 * @param message Optional error message
 * @param stack Optional stack trace
 * @returns
 */
function EthereumJSErrorWithoutCode(message, stack) {
    return new EthereumJSError({ code: exports.DEFAULT_ERROR_CODE }, message, stack);
}
//# sourceMappingURL=errors.js.map