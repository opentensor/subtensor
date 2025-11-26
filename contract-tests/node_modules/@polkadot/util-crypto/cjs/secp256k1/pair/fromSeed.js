"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1PairFromSeed = secp256k1PairFromSeed;
const secp256k1_1 = require("@noble/curves/secp256k1");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name secp256k1PairFromSeed
 * @description Returns a object containing a `publicKey` & `secretKey` generated from the supplied seed.
 */
function secp256k1PairFromSeed(seed, onlyJs) {
    if (seed.length !== 32) {
        throw new Error('Expected valid 32-byte private key as a seed');
    }
    if (!util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())) {
        const full = (0, wasm_crypto_1.secp256k1FromSeed)(seed);
        const publicKey = full.slice(32);
        // There is an issue with the secp256k1 when running in an ASM.js environment where
        // it seems that the lazy static section yields invalid results on the _first_ run.
        // If this happens, fail outright, we cannot allow invalid return values
        // https://github.com/polkadot-js/wasm/issues/307
        if ((0, util_1.u8aEmpty)(publicKey)) {
            throw new Error('Invalid publicKey generated from WASM interface');
        }
        return {
            publicKey,
            secretKey: full.slice(0, 32)
        };
    }
    return {
        publicKey: secp256k1_1.secp256k1.getPublicKey(seed, true),
        secretKey: seed
    };
}
