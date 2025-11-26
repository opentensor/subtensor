"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1Compress = secp256k1Compress;
const secp256k1_1 = require("@noble/curves/secp256k1");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
function secp256k1Compress(publicKey, onlyJs) {
    if (![33, 65].includes(publicKey.length)) {
        throw new Error(`Invalid publicKey provided, received ${publicKey.length} bytes input`);
    }
    if (publicKey.length === 33) {
        return publicKey;
    }
    return !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.secp256k1Compress)(publicKey)
        : secp256k1_1.secp256k1.ProjectivePoint.fromHex(publicKey).toRawBytes(true);
}
