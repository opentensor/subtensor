"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.pbkdf2Encode = pbkdf2Encode;
const pbkdf2_1 = require("@noble/hashes/pbkdf2");
const sha512_1 = require("@noble/hashes/sha512");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const asU8a_js_1 = require("../random/asU8a.js");
function pbkdf2Encode(passphrase, salt = (0, asU8a_js_1.randomAsU8a)(), rounds = 2048, onlyJs) {
    const u8aPass = (0, util_1.u8aToU8a)(passphrase);
    const u8aSalt = (0, util_1.u8aToU8a)(salt);
    return {
        password: !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
            ? (0, wasm_crypto_1.pbkdf2)(u8aPass, u8aSalt, rounds)
            : (0, pbkdf2_1.pbkdf2)(sha512_1.sha512, u8aPass, u8aSalt, { c: rounds, dkLen: 64 }),
        rounds,
        salt
    };
}
