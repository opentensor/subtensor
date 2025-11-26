"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519Verify = sr25519Verify;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name sr25519Verify
 * @description Verifies the signature of `message`, using the supplied pair
 */
function sr25519Verify(message, signature, publicKey) {
    const publicKeyU8a = (0, util_1.u8aToU8a)(publicKey);
    const signatureU8a = (0, util_1.u8aToU8a)(signature);
    if (publicKeyU8a.length !== 32) {
        throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    }
    else if (signatureU8a.length !== 64) {
        throw new Error(`Invalid signature, received ${signatureU8a.length} bytes, expected 64`);
    }
    return (0, wasm_crypto_1.sr25519Verify)(signatureU8a, (0, util_1.u8aToU8a)(message), publicKeyU8a);
}
