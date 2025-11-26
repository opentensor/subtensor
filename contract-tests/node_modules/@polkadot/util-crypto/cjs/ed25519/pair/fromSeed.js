"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ed25519PairFromSeed = ed25519PairFromSeed;
const ed25519_1 = require("@noble/curves/ed25519");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name ed25519PairFromSeed
 * @summary Creates a new public/secret keypair from a seed.
 * @description
 * Returns a object containing a `publicKey` & `secretKey` generated from the supplied seed.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519PairFromSeed } from '@polkadot/util-crypto';
 *
 * ed25519PairFromSeed(...); // => { secretKey: [...], publicKey: [...] }
 * ```
 */
function ed25519PairFromSeed(seed, onlyJs) {
    if (!util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())) {
        const full = (0, wasm_crypto_1.ed25519KeypairFromSeed)(seed);
        return {
            publicKey: full.slice(32),
            secretKey: full.slice(0, 64)
        };
    }
    const publicKey = ed25519_1.ed25519.getPublicKey(seed);
    return {
        publicKey,
        secretKey: (0, util_1.u8aConcatStrict)([seed, publicKey])
    };
}
