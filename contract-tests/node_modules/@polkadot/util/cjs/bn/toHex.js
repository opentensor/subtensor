"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bnToHex = bnToHex;
const index_js_1 = require("../u8a/index.js");
const toU8a_js_1 = require("./toU8a.js");
/**
 * @name bnToHex
 * @summary Creates a hex value from a BN.js bignumber object.
 * @description
 * `null` inputs returns a `0x` result, BN values return the actual value as a `0x` prefixed hex value. Anything that is not a BN object throws an error. With `bitLength` set, it fixes the number to the specified length.
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnToHex } from '@polkadot/util';
 *
 * bnToHex(new BN(0x123456)); // => '0x123456'
 * ```
 */
function bnToHex(value, { bitLength = -1, isLe = false, isNegative = false } = {}) {
    return (0, index_js_1.u8aToHex)((0, toU8a_js_1.bnToU8a)(value, { bitLength, isLe, isNegative }));
}
