"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519PairFromSeed = sr25519PairFromSeed;
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
const fromU8a_js_1 = require("./fromU8a.js");
/**
 * @name sr25519PairFromSeed
 * @description Returns a object containing a `publicKey` & `secretKey` generated from the supplied seed.
 */
function sr25519PairFromSeed(seed) {
    const seedU8a = (0, util_1.u8aToU8a)(seed);
    if (seedU8a.length !== 32) {
        throw new Error(`Expected a seed matching 32 bytes, found ${seedU8a.length}`);
    }
    return (0, fromU8a_js_1.sr25519PairFromU8a)((0, wasm_crypto_1.sr25519KeypairFromSeed)(seedU8a));
}
