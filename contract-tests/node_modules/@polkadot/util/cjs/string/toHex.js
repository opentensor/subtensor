"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stringToHex = stringToHex;
const toHex_js_1 = require("../u8a/toHex.js");
const toU8a_js_1 = require("./toU8a.js");
/**
 * @name stringToHex
 * @summary Creates a hex string from a utf-8 string
 * @description
 * String input values return the actual encoded hex value.
 * @example
 * <BR>
 *
 * ```javascript
 * import { stringToHex } from '@polkadot/util';
 *
 * stringToU8a('hello'); // 0x68656c6c6f
 * ```
 */
function stringToHex(value) {
    return (0, toHex_js_1.u8aToHex)((0, toU8a_js_1.stringToU8a)(value));
}
