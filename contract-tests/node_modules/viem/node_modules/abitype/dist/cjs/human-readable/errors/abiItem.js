"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.UnknownSolidityTypeError = exports.UnknownTypeError = exports.InvalidAbiItemError = void 0;
const errors_js_1 = require("../../errors.js");
class InvalidAbiItemError extends errors_js_1.BaseError {
    constructor({ signature }) {
        super('Failed to parse ABI item.', {
            details: `parseAbiItem(${JSON.stringify(signature, null, 2)})`,
            docsPath: '/api/human#parseabiitem-1',
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'InvalidAbiItemError'
        });
    }
}
exports.InvalidAbiItemError = InvalidAbiItemError;
class UnknownTypeError extends errors_js_1.BaseError {
    constructor({ type }) {
        super('Unknown type.', {
            metaMessages: [
                `Type "${type}" is not a valid ABI type. Perhaps you forgot to include a struct signature?`,
            ],
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'UnknownTypeError'
        });
    }
}
exports.UnknownTypeError = UnknownTypeError;
class UnknownSolidityTypeError extends errors_js_1.BaseError {
    constructor({ type }) {
        super('Unknown type.', {
            metaMessages: [`Type "${type}" is not a valid ABI type.`],
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'UnknownSolidityTypeError'
        });
    }
}
exports.UnknownSolidityTypeError = UnknownSolidityTypeError;
//# sourceMappingURL=abiItem.js.map