"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hexToString = hexToString;
const toString_js_1 = require("../u8a/toString.js");
const toU8a_js_1 = require("./toU8a.js");
/**
 * @name hexToU8a
 * @summary Creates a Uint8Array object from a hex string.
 * @description
 * Hex input values return the actual bytes value converted to a string. Anything that is not a hex string (including the `0x` prefix) throws an error.
 * @example
 * <BR>
 *
 * ```javascript
 * import { hexToString } from '@polkadot/util';
 *
 * hexToU8a('0x68656c6c6f'); // hello
 * ```
 */
function hexToString(_value) {
    return (0, toString_js_1.u8aToString)((0, toU8a_js_1.hexToU8a)(_value));
}
