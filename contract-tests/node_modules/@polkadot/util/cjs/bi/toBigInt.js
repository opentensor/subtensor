"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nToBigInt = nToBigInt;
const x_bigint_1 = require("@polkadot/x-bigint");
const toBigInt_js_1 = require("../hex/toBigInt.js");
const bn_js_1 = require("../is/bn.js");
const hex_js_1 = require("../is/hex.js");
const toBigInt_js_2 = require("../is/toBigInt.js");
const toBn_js_1 = require("../is/toBn.js");
/**
 * @name nToBigInt
 * @summary Creates a bigInt value from a BN, bigint, string (base 10 or hex) or number input.
 */
function nToBigInt(value) {
    return typeof value === 'bigint'
        ? value
        : !value
            ? (0, x_bigint_1.BigInt)(0)
            : (0, hex_js_1.isHex)(value)
                ? (0, toBigInt_js_1.hexToBigInt)(value.toString())
                : (0, bn_js_1.isBn)(value)
                    ? (0, x_bigint_1.BigInt)(value.toString())
                    : (0, toBigInt_js_2.isToBigInt)(value)
                        ? value.toBigInt()
                        : (0, toBn_js_1.isToBn)(value)
                            ? (0, x_bigint_1.BigInt)(value.toBn().toString())
                            : (0, x_bigint_1.BigInt)(value);
}
