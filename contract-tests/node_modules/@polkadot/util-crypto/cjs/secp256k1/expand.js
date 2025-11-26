"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1Expand = secp256k1Expand;
const secp256k1_1 = require("@noble/curves/secp256k1");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const bn_js_1 = require("../bn.js");
function secp256k1Expand(publicKey, onlyJs) {
    if (![33, 65].includes(publicKey.length)) {
        throw new Error(`Invalid publicKey provided, received ${publicKey.length} bytes input`);
    }
    if (publicKey.length === 65) {
        return publicKey.subarray(1);
    }
    if (!util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())) {
        return (0, wasm_crypto_1.secp256k1Expand)(publicKey).subarray(1);
    }
    const { px, py } = secp256k1_1.secp256k1.ProjectivePoint.fromHex(publicKey);
    return (0, util_1.u8aConcat)((0, util_1.bnToU8a)(px, bn_js_1.BN_BE_256_OPTS), (0, util_1.bnToU8a)(py, bn_js_1.BN_BE_256_OPTS));
}
