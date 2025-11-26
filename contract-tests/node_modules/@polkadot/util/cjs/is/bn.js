"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBn = isBn;
const bn_js_1 = require("../bn/bn.js");
/**
 * @name isBn
 * @summary Tests for a `BN` object instance.
 * @description
 * Checks to see if the input object is an instance of `BN` (bn.js).
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { isBn } from '@polkadot/util';
 *
 * console.log('isBn', isBn(new BN(1))); // => true
 * ```
 */
function isBn(value) {
    return bn_js_1.BN.isBN(value);
}
