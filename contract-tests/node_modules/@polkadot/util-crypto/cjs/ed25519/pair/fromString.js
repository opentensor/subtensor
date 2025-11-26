"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ed25519PairFromString = ed25519PairFromString;
const util_1 = require("@polkadot/util");
const asU8a_js_1 = require("../../blake2/asU8a.js");
const fromSeed_js_1 = require("./fromSeed.js");
/**
 * @name ed25519PairFromString
 * @summary Creates a new public/secret keypair from a string.
 * @description
 * Returns a object containing a `publicKey` & `secretKey` generated from the supplied string. The string is hashed and the value used as the input seed.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519PairFromString } from '@polkadot/util-crypto';
 *
 * ed25519PairFromString('test'); // => { secretKey: [...], publicKey: [...] }
 * ```
 */
function ed25519PairFromString(value) {
    return (0, fromSeed_js_1.ed25519PairFromSeed)((0, asU8a_js_1.blake2AsU8a)((0, util_1.stringToU8a)(value)));
}
