"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hmacSha512AsU8a = exports.hmacSha256AsU8a = void 0;
exports.hmacShaAsU8a = hmacShaAsU8a;
const hmac_1 = require("@noble/hashes/hmac");
const sha256_1 = require("@noble/hashes/sha256");
const sha512_1 = require("@noble/hashes/sha512");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const JS_HASH = {
    256: sha256_1.sha256,
    512: sha512_1.sha512
};
const WA_MHAC = {
    256: wasm_crypto_1.hmacSha256,
    512: wasm_crypto_1.hmacSha512
};
function createSha(bitLength) {
    return (key, data, onlyJs) => hmacShaAsU8a(key, data, bitLength, onlyJs);
}
/**
 * @name hmacShaAsU8a
 * @description creates a Hmac Sha (256/512) Uint8Array from the key & data
 */
function hmacShaAsU8a(key, data, bitLength = 256, onlyJs) {
    const u8aKey = (0, util_1.u8aToU8a)(key);
    return !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
        ? WA_MHAC[bitLength](u8aKey, data)
        : (0, hmac_1.hmac)(JS_HASH[bitLength], u8aKey, data);
}
/**
 * @name hmacSha256AsU8a
 * @description creates a Hmac Sha256 Uint8Array from the key & data
 */
exports.hmacSha256AsU8a = createSha(256);
/**
 * @name hmacSha512AsU8a
 * @description creates a Hmac Sha512 Uint8Array from the key & data
 */
exports.hmacSha512AsU8a = createSha(512);
