"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.scryptEncode = scryptEncode;
const scrypt_1 = require("@noble/hashes/scrypt");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const asU8a_js_1 = require("../random/asU8a.js");
const defaults_js_1 = require("./defaults.js");
function scryptEncode(passphrase, salt = (0, asU8a_js_1.randomAsU8a)(), params = defaults_js_1.DEFAULT_PARAMS, onlyJs) {
    const u8a = (0, util_1.u8aToU8a)(passphrase);
    return {
        params,
        password: !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
            ? (0, wasm_crypto_1.scrypt)(u8a, salt, Math.log2(params.N), params.r, params.p)
            : (0, scrypt_1.scrypt)(u8a, salt, (0, util_1.objectSpread)({ dkLen: 64 }, params)),
        salt
    };
}
