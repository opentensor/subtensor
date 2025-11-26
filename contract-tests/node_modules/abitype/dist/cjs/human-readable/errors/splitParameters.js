"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.InvalidParenthesisError = void 0;
const errors_js_1 = require("../../errors.js");
class InvalidParenthesisError extends errors_js_1.BaseError {
    constructor({ current, depth }) {
        super('Unbalanced parentheses.', {
            metaMessages: [
                `"${current.trim()}" has too many ${depth > 0 ? 'opening' : 'closing'} parentheses.`,
            ],
            details: `Depth "${depth}"`,
        });
        Object.defineProperty(this, "name", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: 'InvalidParenthesisError'
        });
    }
}
exports.InvalidParenthesisError = InvalidParenthesisError;
//# sourceMappingURL=splitParameters.js.map