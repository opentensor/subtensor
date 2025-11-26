"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bnToBn = bnToBn;
const toBn_js_1 = require("../hex/toBn.js");
const bigInt_js_1 = require("../is/bigInt.js");
const hex_js_1 = require("../is/hex.js");
const toBigInt_js_1 = require("../is/toBigInt.js");
const toBn_js_2 = require("../is/toBn.js");
const bn_js_1 = require("./bn.js");
/**
 * @name bnToBn
 * @summary Creates a BN value from a BN, bigint, string (base 10 or hex) or number input.
 * @description
 * `null` inputs returns a `0x0` result, BN values returns the value, numbers returns a BN representation.
 * @example
 * <BR>
 *
 * ```javascript
 * import BN from 'bn.js';
 * import { bnToBn } from '@polkadot/util';
 *
 * bnToBn(0x1234); // => BN(0x1234)
 * bnToBn(new BN(0x1234)); // => BN(0x1234)
 * ```
 */
function bnToBn(value) {
    return value
        ? bn_js_1.BN.isBN(value)
            ? value
            : (0, hex_js_1.isHex)(value)
                ? (0, toBn_js_1.hexToBn)(value.toString())
                : (0, bigInt_js_1.isBigInt)(value)
                    ? new bn_js_1.BN(value.toString())
                    : (0, toBn_js_2.isToBn)(value)
                        ? value.toBn()
                        : (0, toBigInt_js_1.isToBigInt)(value)
                            ? new bn_js_1.BN(value.toBigInt().toString())
                            : new bn_js_1.BN(value)
        : new bn_js_1.BN(0);
}
