"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1Recover = secp256k1Recover;
const secp256k1_1 = require("@noble/curves/secp256k1");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const compress_js_1 = require("./compress.js");
const expand_js_1 = require("./expand.js");
/**
 * @name secp256k1Recover
 * @description Recovers a publicKey from the supplied signature
 */
function secp256k1Recover(msgHash, signature, recovery, hashType = 'blake2', onlyJs) {
    const sig = (0, util_1.u8aToU8a)(signature).subarray(0, 64);
    const msg = (0, util_1.u8aToU8a)(msgHash);
    const publicKey = !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.secp256k1Recover)(msg, sig, recovery)
        : secp256k1_1.secp256k1.Signature
            .fromCompact(sig)
            .addRecoveryBit(recovery)
            .recoverPublicKey(msg)
            .toRawBytes();
    if (!publicKey) {
        throw new Error('Unable to recover publicKey from signature');
    }
    return hashType === 'keccak'
        ? (0, expand_js_1.secp256k1Expand)(publicKey, onlyJs)
        : (0, compress_js_1.secp256k1Compress)(publicKey, onlyJs);
}
