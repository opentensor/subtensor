"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nToU8a = nToU8a;
const x_bigint_1 = require("@polkadot/x-bigint");
const consts_js_1 = require("./consts.js");
const toBigInt_js_1 = require("./toBigInt.js");
const DIV = (0, x_bigint_1.BigInt)(256);
const NEG_MASK = (0, x_bigint_1.BigInt)(0xff);
function toU8a(value, isLe, isNegative) {
    const arr = [];
    const withSigned = isNegative && (value < consts_js_1._0n);
    if (withSigned) {
        value = (value + consts_js_1._1n) * -consts_js_1._1n;
    }
    while (value !== consts_js_1._0n) {
        const mod = value % DIV;
        const val = Number(withSigned
            ? mod ^ NEG_MASK
            : mod);
        if (isLe) {
            arr.push(val);
        }
        else {
            arr.unshift(val);
        }
        value = (value - mod) / DIV;
    }
    return Uint8Array.from(arr);
}
/**
 * @name nToU8a
 * @summary Creates a Uint8Array object from a bigint.
 */
function nToU8a(value, { bitLength = -1, isLe = true, isNegative = false } = {}) {
    const valueBi = (0, toBigInt_js_1.nToBigInt)(value);
    if (valueBi === consts_js_1._0n) {
        return bitLength === -1
            ? new Uint8Array(1)
            : new Uint8Array(Math.ceil((bitLength || 0) / 8));
    }
    const u8a = toU8a(valueBi, isLe, isNegative);
    if (bitLength === -1) {
        return u8a;
    }
    const byteLength = Math.ceil((bitLength || 0) / 8);
    const output = new Uint8Array(byteLength);
    if (isNegative) {
        output.fill(0xff);
    }
    output.set(u8a, isLe ? 0 : byteLength - u8a.length);
    return output;
}
