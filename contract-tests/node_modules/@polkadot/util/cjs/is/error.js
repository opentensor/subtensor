"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isError = isError;
/**
 * @name isError
 * @summary Tests for a `Error` object instance.
 * @description
 * Checks to see if the input object is an instance of `Error`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { isError } from '@polkadot/util';
 *
 * console.log('isError', isError(new Error('message'))); // => true
 * ```
 */
function isError(value) {
    return (((value && value.constructor) === Error) ||
        value instanceof Error);
}
