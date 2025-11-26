"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1Sign = secp256k1Sign;
const secp256k1_1 = require("@noble/curves/secp256k1");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const bn_js_1 = require("../bn.js");
const hasher_js_1 = require("./hasher.js");
/**
 * @name secp256k1Sign
 * @description Returns message signature of `message`, using the supplied pair
 */
function secp256k1Sign(message, { secretKey }, hashType = 'blake2', onlyJs) {
    if (secretKey?.length !== 32) {
        throw new Error('Expected valid secp256k1 secretKey, 32-bytes');
    }
    const data = (0, hasher_js_1.hasher)(hashType, message, onlyJs);
    if (!util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())) {
        return (0, wasm_crypto_1.secp256k1Sign)(data, secretKey);
    }
    const signature = secp256k1_1.secp256k1.sign(data, secretKey, { lowS: true });
    return (0, util_1.u8aConcat)((0, util_1.bnToU8a)(signature.r, bn_js_1.BN_BE_256_OPTS), (0, util_1.bnToU8a)(signature.s, bn_js_1.BN_BE_256_OPTS), new Uint8Array([signature.recovery || 0]));
}
