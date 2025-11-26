"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBigInt = isBigInt;
/**
 * @name isBigInt
 * @summary Tests for a `BigInt` object instance.
 * @description
 * Checks to see if the input object is an instance of `BigInt`
 * @example
 * <BR>
 *
 * ```javascript
 * import { isBigInt } from '@polkadot/util';
 *
 * console.log('isBigInt', isBigInt(123_456n)); // => true
 * ```
 */
function isBigInt(value) {
    return typeof value === 'bigint';
}
