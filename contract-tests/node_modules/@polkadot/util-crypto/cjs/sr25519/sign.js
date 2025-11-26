"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519Sign = sr25519Sign;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name sr25519Sign
 * @description Returns message signature of `message`, using the supplied pair
 */
function sr25519Sign(message, { publicKey, secretKey }) {
    if (publicKey?.length !== 32) {
        throw new Error('Expected a valid publicKey, 32-bytes');
    }
    else if (secretKey?.length !== 64) {
        throw new Error('Expected a valid secretKey, 64-bytes');
    }
    return (0, wasm_crypto_1.sr25519Sign)(publicKey, secretKey, (0, util_1.u8aToU8a)(message));
}
