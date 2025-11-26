"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.assert = assert;
exports.assertReturn = assertReturn;
exports.assertUnreachable = assertUnreachable;
const function_js_1 = require("./is/function.js");
/**
 * @name assert
 * @summary Checks for a valid test, if not Error is thrown.
 * @description
 * Checks that `test` is a truthy value. If value is falsy (`null`, `undefined`, `false`, ...), it throws an Error with the supplied `message`. When `test` passes, `true` is returned.
 * @example
 * <BR>
 *
 * ```javascript
 * const { assert } from '@polkadot/util';
 *
 * assert(true, 'True should be true'); // passes
 * assert(false, 'False should not be true'); // Error thrown
 * assert(false, () => 'message'); // Error with 'message'
 * ```
 */
function assert(condition, message) {
    if (!condition) {
        throw new Error((0, function_js_1.isFunction)(message)
            ? message()
            : message);
    }
}
/**
 * @name assertReturn
 * @description Returns when the value is not undefined/null, otherwise throws assertion error
 */
function assertReturn(value, message) {
    assert(value !== undefined && value !== null, message);
    return value;
}
/**
 * @name assertUnreachable
 * @description An assertion helper that ensures all codepaths are followed
 */
function assertUnreachable(x) {
    throw new Error(`This codepath should be unreachable. Unhandled input: ${x}`);
}
