"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ed25519PairFromSecret = ed25519PairFromSecret;
/**
 * @name ed25519PairFromSecret
 * @summary Creates a new public/secret keypair from a secret.
 * @description
 * Returns a object containing a `publicKey` & `secretKey` generated from the supplied secret.
 * @example
 * <BR>
 *
 * ```javascript
 * import { ed25519PairFromSecret } from '@polkadot/util-crypto';
 *
 * ed25519PairFromSecret(...); // => { secretKey: [...], publicKey: [...] }
 * ```
 */
function ed25519PairFromSecret(secretKey) {
    if (secretKey.length !== 64) {
        throw new Error('Invalid secretKey provided');
    }
    return {
        publicKey: secretKey.slice(32),
        secretKey
    };
}
