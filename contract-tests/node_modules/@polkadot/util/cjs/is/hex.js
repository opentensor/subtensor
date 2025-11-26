"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.REGEX_HEX_NOPREFIX = exports.REGEX_HEX_PREFIXED = void 0;
exports.isHex = isHex;
exports.REGEX_HEX_PREFIXED = /^0x[\da-fA-F]+$/;
exports.REGEX_HEX_NOPREFIX = /^[\da-fA-F]+$/;
/**
 * @name isHex
 * @summary Tests for a hex string.
 * @description
 * Checks to see if the input value is a `0x` prefixed hex string. Optionally (`bitLength` !== -1) checks to see if the bitLength is correct.
 * @example
 * <BR>
 *
 * ```javascript
 * import { isHex } from '@polkadot/util';
 *
 * isHex('0x1234'); // => true
 * isHex('0x1234', 8); // => false
 * ```
 */
function isHex(value, bitLength = -1, ignoreLength) {
    return (typeof value === 'string' && (value === '0x' ||
        exports.REGEX_HEX_PREFIXED.test(value))) && (bitLength === -1
        ? (ignoreLength || (value.length % 2 === 0))
        : (value.length === (2 + Math.ceil(bitLength / 4))));
}
