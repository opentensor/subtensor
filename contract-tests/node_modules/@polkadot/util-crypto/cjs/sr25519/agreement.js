"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519Agreement = sr25519Agreement;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name sr25519Agreement
 * @description Key agreement between other's public key and self secret key
 */
function sr25519Agreement(secretKey, publicKey) {
    const secretKeyU8a = (0, util_1.u8aToU8a)(secretKey);
    const publicKeyU8a = (0, util_1.u8aToU8a)(publicKey);
    if (publicKeyU8a.length !== 32) {
        throw new Error(`Invalid publicKey, received ${publicKeyU8a.length} bytes, expected 32`);
    }
    else if (secretKeyU8a.length !== 64) {
        throw new Error(`Invalid secretKey, received ${secretKeyU8a.length} bytes, expected 64`);
    }
    return (0, wasm_crypto_1.sr25519Agree)(publicKeyU8a, secretKeyU8a);
}
