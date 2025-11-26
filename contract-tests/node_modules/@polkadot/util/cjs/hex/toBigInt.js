"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hexToBigInt = hexToBigInt;
const x_bigint_1 = require("@polkadot/x-bigint");
const toBigInt_js_1 = require("../u8a/toBigInt.js");
const toU8a_js_1 = require("./toU8a.js");
/**
 * @name hexToBigInt
 * @summary Creates a BigInt instance object from a hex string.
 */
function hexToBigInt(value, { isLe = false, isNegative = false } = {}) {
    return !value || value === '0x'
        ? (0, x_bigint_1.BigInt)(0)
        : (0, toBigInt_js_1.u8aToBigInt)((0, toU8a_js_1.hexToU8a)(value), { isLe, isNegative });
}
