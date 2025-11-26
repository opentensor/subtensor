"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ed25519Verify = ed25519Verify;
const ed25519_1 = require("@noble/curves/ed25519");
const util_1 = require("@polkadot/util");
const wasm_crypto_1 = require("@polkadot/wasm-crypto");
/**
 * @name ed25519Sign
 * @summary Verifies the signature on the supplied message.
 * @description
 * Verifies the `signature` on `message` with the supplied `publicKey`. Returns `true` on sucess, `false` otherwise.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519Verify } from '@polkadot/util-crypto';
 *
 * ed25519Verify([...], [...], [...]); // => true/false
 * ```
 */
function ed25519Verify(message, signature, publicKey, onlyJs) {
    const messageU8a = (0, util_1.u8aToU8a)(message);
    const publicKeyU8a = (0, util_1.u8aToU8a)(publicKey);
    const signatureU8a = (0, util_1.u8aToU8a)(signature);
    if (publicKeyU8a.length !== 32) {
        throw new Error(`Invalid publicKey, received ${publicKeyU8a.length}, expected 32`);
    }
    else if (signatureU8a.length !== 64) {
        throw new Error(`Invalid signature, received ${signatureU8a.length} bytes, expected 64`);
    }
    try {
        return !util_1.hasBigInt || (!onlyJs && (0, wasm_crypto_1.isReady)())
            ? (0, wasm_crypto_1.ed25519Verify)(signatureU8a, messageU8a, publicKeyU8a)
            : ed25519_1.ed25519.verify(signatureU8a, messageU8a, publicKeyU8a);
    }
    catch {
        return false;
    }
}
