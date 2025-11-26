"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.InvalidStructSignatureError = exports.UnknownSignatureError = exports.InvalidSignatureError = void 0;
const errors_js_1 = require("../../errors.js");
class InvalidSignatureError extends errors_js_1.BaseError {
    constructor({ signature, type, }) {
        super(`Invalid ${type} signature.`, {
            details: signature,
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'InvalidSignatureError'
        });
    }
}
exports.InvalidSignatureError = InvalidSignatureError;
class UnknownSignatureError extends errors_js_1.BaseError {
    constructor({ signature }) {
        super('Unknown signature.', {
            details: signature,
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'UnknownSignatureError'
        });
    }
}
exports.UnknownSignatureError = UnknownSignatureError;
class InvalidStructSignatureError extends errors_js_1.BaseError {
    constructor({ signature }) {
        super('Invalid struct signature.', {
            details: signature,
            metaMessages: ['No properties exist.'],
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'InvalidStructSignatureError'
        });
    }
}
exports.InvalidStructSignatureError = InvalidStructSignatureError;
//# sourceMappingURL=signature.js.map