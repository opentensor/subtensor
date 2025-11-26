"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ed25519Sign = ed25519Sign;
const ed25519_1 = require("@noble/curves/ed25519");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name ed25519Sign
 * @summary Signs a message using the supplied secretKey
 * @description
 * Returns message signature of `message`, using the `secretKey`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519Sign } from '@polkadot/util-crypto';
 *
 * ed25519Sign([...], [...]); // => [...]
 * ```
 */
function ed25519Sign(message, { publicKey, secretKey }, onlyJs) {
    if (!secretKey) {
        throw new Error('Expected a valid secretKey');
    }
    else if (!publicKey) {
        throw new Error('Expected a valid publicKey');
    }
    const messageU8a = (0, util_1.u8aToU8a)(message);
    const privateU8a = secretKey.subarray(0, 32);
    return !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
        ? (0, wasm_crypto_1.ed25519Sign)(publicKey, privateU8a, messageU8a)
        : ed25519_1.ed25519.sign(messageU8a, privateU8a);
}
