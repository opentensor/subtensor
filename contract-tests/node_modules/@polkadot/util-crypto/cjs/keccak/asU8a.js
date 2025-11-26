"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.keccakAsHex = exports.keccak512AsU8a = exports.keccak256AsU8a = exports.keccakAsU8a = void 0;
const sha3_1 = require("@noble/hashes/sha3");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const helpers_js_1 = require("../helpers.js");
/**
 * @name keccakAsU8a
 * @summary Creates a keccak Uint8Array from the input.
 * @description
 * From either a `string` or a `Buffer` input, create the keccak and return the result as a `Uint8Array`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { keccakAsU8a } from '@polkadot/util-crypto';
 *
 * keccakAsU8a('123'); // => Uint8Array
 * ```
 */
exports.keccakAsU8a = (0, helpers_js_1.createDualHasher)({ 256: wasm_crypto_1.keccak256, 512: wasm_crypto_1.keccak512 }, { 256: sha3_1.keccak_256, 512: sha3_1.keccak_512 });
/**
 * @name keccak256AsU8a
 * @description Creates a keccak256 Uint8Array from the input.
 */
exports.keccak256AsU8a = (0, helpers_js_1.createBitHasher)(256, exports.keccakAsU8a);
/**
 * @name keccak512AsU8a
 * @description Creates a keccak512 Uint8Array from the input.
 */
exports.keccak512AsU8a = (0, helpers_js_1.createBitHasher)(512, exports.keccakAsU8a);
/**
 * @name keccakAsHex
 * @description Creates a keccak hex string from the input.
 */
exports.keccakAsHex = (0, helpers_js_1.createAsHex)(exports.keccakAsU8a);
