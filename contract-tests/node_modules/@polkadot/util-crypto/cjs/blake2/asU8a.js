"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.blake2AsHex = void 0;
exports.blake2AsU8a = blake2AsU8a;
const blake2b_1 = require("@noble/hashes/blake2b");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const helpers_js_1 = require("../helpers.js");
/**
 * @name blake2AsU8a
 * @summary Creates a blake2b u8a from the input.
 * @description
 * From a `Uint8Array` input, create the blake2b and return the result as a u8a with the specified `bitLength`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { blake2AsU8a } from '@polkadot/util-crypto';
 *
 * blake2AsU8a('abc'); // => [0xba, 0x80, 0xa5, 0x3f, 0x98, 0x1c, 0x4d, 0x0d]
 * ```
 */
function blake2AsU8a(data, bitLength = 256, key, onlyJs) {
    const byteLength = Math.ceil(bitLength / 8);
    const u8a = (0, util_1.u8aToU8a)(data);
    return !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.blake2b)(u8a, (0, util_1.u8aToU8a)(key), byteLength)
        : key
            ? (0, blake2b_1.blake2b)(u8a, { dkLen: byteLength, key })
            : (0, blake2b_1.blake2b)(u8a, { dkLen: byteLength });
}
/**
 * @name blake2AsHex
 * @description Creates a blake2b hex from the input.
 */
exports.blake2AsHex = (0, helpers_js_1.createAsHex)(blake2AsU8a);
