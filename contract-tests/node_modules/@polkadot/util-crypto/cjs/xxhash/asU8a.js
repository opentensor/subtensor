"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xxhashAsHex = void 0;
exports.xxhashAsU8a = xxhashAsU8a;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const helpers_js_1 = require("../helpers.js");
const xxhash64_js_1 = require("./xxhash64.js");
/**
 * @name xxhashAsU8a
 * @summary Creates a xxhash64 u8a from the input.
 * @description
 * From either a `string`, `Uint8Array` or a `Buffer` input, create the xxhash64 and return the result as a `Uint8Array` with the specified `bitLength`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { xxhashAsU8a } from '@polkadot/util-crypto';
 *
 * xxhashAsU8a('abc'); // => 0x44bc2cf5ad770999
 * ```
 */
function xxhashAsU8a(data, bitLength = 64, onlyJs) {
    const rounds = Math.ceil(bitLength / 64);
    const u8a = (0, util_1.u8aToU8a)(data);
    if (!util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())) {
        return (0, wasm_crypto_1.twox)(u8a, rounds);
    }
    const result = new Uint8Array(rounds * 8);
    for (let seed = 0; seed < rounds; seed++) {
        result.set((0, xxhash64_js_1.xxhash64)(u8a, seed).reverse(), seed * 8);
    }
    return result;
}
/**
 * @name xxhashAsHex
 * @description Creates a xxhash64 hex from the input.
 */
exports.xxhashAsHex = (0, helpers_js_1.createAsHex)(xxhashAsU8a);
