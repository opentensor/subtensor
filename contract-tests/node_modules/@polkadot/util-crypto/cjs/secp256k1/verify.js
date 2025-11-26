"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1Verify = secp256k1Verify;
const util_1 = require("@polkadot/util");
const hasher_js_1 = require("./hasher.js");
const recover_js_1 = require("./recover.js");
/**
 * @name secp256k1Verify
 * @description Verifies the signature of `message`, using the supplied pair
 */
function secp256k1Verify(msgHash, signature, address, hashType = 'blake2', onlyJs) {
    const sig = (0, util_1.u8aToU8a)(signature);
    if (sig.length !== 65) {
        throw new Error(`Expected signature with 65 bytes, ${sig.length} found instead`);
    }
    const publicKey = (0, recover_js_1.secp256k1Recover)((0, hasher_js_1.hasher)(hashType, msgHash), sig, sig[64], hashType, onlyJs);
    const signerAddr = (0, hasher_js_1.hasher)(hashType, publicKey, onlyJs);
    const inputAddr = (0, util_1.u8aToU8a)(address);
    // for Ethereum (keccak) the last 20 bytes is the address
    return (0, util_1.u8aEq)(publicKey, inputAddr) || (hashType === 'keccak'
        ? (0, util_1.u8aEq)(signerAddr.slice(-20), inputAddr.slice(-20))
        : (0, util_1.u8aEq)(signerAddr, inputAddr));
}
