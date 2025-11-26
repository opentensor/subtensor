"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.numberToU8a = numberToU8a;
const toU8a_js_1 = require("../hex/toU8a.js");
const toHex_js_1 = require("./toHex.js");
/**
 * @name numberToU8a
 * @summary Creates a Uint8Array object from a number.
 * @description
 * `null`/`undefined`/`NaN` inputs returns an empty `Uint8Array` result. `number` input values return the actual bytes value converted to a `Uint8Array`. With `bitLength`, it converts the value to the equivalent size.
 * @example
 * <BR>
 *
 * ```javascript
 * import { numberToU8a } from '@polkadot/util';
 *
 * numberToU8a(0x1234); // => [0x12, 0x34]
 * ```
 */
function numberToU8a(value, bitLength = -1) {
    return (0, toU8a_js_1.hexToU8a)((0, toHex_js_1.numberToHex)(value, bitLength));
}
