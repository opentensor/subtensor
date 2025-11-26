import { randomAsU8a } from '../random/asU8a.js';
import { naclSecretbox } from './tweetnacl.js';
/**
 * @name naclEncrypt
 * @summary Encrypts a message using the supplied secretKey and nonce
 * @description
 * Returns an encrypted message, using the `secretKey` and `nonce`. If the `nonce` was not supplied, a random value is generated.
 * @example
 * <BR>
 *
 * ```javascript
 * import { naclEncrypt } from '@polkadot/util-crypto';
 *
 * naclEncrypt([...], [...]); // => [...]
 * ```
 */
export function naclEncrypt(message, secret, nonce = randomAsU8a(24)) {
    return {
        encrypted: naclSecretbox(message, nonce, secret),
        nonce
    };
}
