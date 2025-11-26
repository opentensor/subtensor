"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sha512AsU8a = exports.sha256AsU8a = exports.shaAsU8a = void 0;
const sha256_1 = require("@noble/hashes/sha256");
const sha512_1 = require("@noble/hashes/sha512");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const helpers_js_1 = require("../helpers.js");
/**
 * @name shaAsU8a
 * @summary Creates a sha Uint8Array from the input.
 */
exports.shaAsU8a = (0, helpers_js_1.createDualHasher)({ 256: wasm_crypto_1.sha256, 512: wasm_crypto_1.sha512 }, { 256: sha256_1.sha256, 512: sha512_1.sha512 });
/**
 * @name sha256AsU8a
 * @summary Creates a sha256 Uint8Array from the input.
 */
exports.sha256AsU8a = (0, helpers_js_1.createBitHasher)(256, exports.shaAsU8a);
/**
 * @name sha512AsU8a
 * @summary Creates a sha512 Uint8Array from the input.
 */
exports.sha512AsU8a = (0, helpers_js_1.createBitHasher)(512, exports.shaAsU8a);
