"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const util_1 = require("@polkadot/util");
const UNKNOWN = -99999;
function extend(that, name, value) {
    Object.defineProperty(that, name, {
        configurable: true,
        enumerable: false,
        value
    });
}
/**
 * @name RpcError
 * @summary Extension to the basic JS Error.
 * @description
 * The built-in JavaScript Error class is extended by adding a code to allow for Error categorization. In addition to the normal `stack`, `message`, the numeric `code` and `data` (any types) parameters are available on the object.
 * @example
 * <BR>
 *
 * ```javascript
 * const { RpcError } from '@polkadot/util');
 *
 * throw new RpcError('some message', RpcError.CODES.METHOD_NOT_FOUND); // => error.code = -32601
 * ```
 */
class RpcError extends Error {
    code;
    data;
    message;
    name;
    stack;
    constructor(message = '', code = UNKNOWN, data) {
        super();
        extend(this, 'message', String(message));
        extend(this, 'name', this.constructor.name);
        extend(this, 'data', data);
        extend(this, 'code', code);
        if ((0, util_1.isFunction)(Error.captureStackTrace)) {
            Error.captureStackTrace(this, this.constructor);
        }
        else {
            const { stack } = new Error(message);
            stack && extend(this, 'stack', stack);
        }
    }
    static CODES = {
        ASSERT: -90009,
        INVALID_JSONRPC: -99998,
        METHOD_NOT_FOUND: -32601, // Rust client
        UNKNOWN
    };
}
exports.default = RpcError;
