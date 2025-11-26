import { naclSecretboxOpen } from './tweetnacl.js';
/**
 * @name naclDecrypt
 * @summary Decrypts a message using the supplied secretKey and nonce
 * @description
 * Returns an decrypted message, using the `secret` and `nonce`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { naclDecrypt } from '@polkadot/util-crypto';
 *
 * naclDecrypt([...], [...], [...]); // => [...]
 * ```
 */
export function naclDecrypt(encrypted, nonce, secret) {
    return naclSecretboxOpen(encrypted, nonce, secret);
}
