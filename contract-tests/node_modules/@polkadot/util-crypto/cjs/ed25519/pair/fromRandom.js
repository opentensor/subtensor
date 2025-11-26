"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ed25519PairFromRandom = ed25519PairFromRandom;
const index_js_1 = require("../../random/index.js");
const fromSeed_js_1 = require("./fromSeed.js");
/**
 * @name ed25519PairFromRandom
 * @summary Creates a new public/secret keypair.
 * @description
 * Returns a new generate object containing a `publicKey` & `secretKey`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519PairFromRandom } from '@polkadot/util-crypto';
 *
 * ed25519PairFromRandom(); // => { secretKey: [...], publicKey: [...] }
 * ```
 */
function ed25519PairFromRandom() {
    return (0, fromSeed_js_1.ed25519PairFromSeed)((0, index_js_1.randomAsU8a)());
}
