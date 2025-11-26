import { x25519 } from '@noble/curves/ed25519';
import * as Bytes from './Bytes.js';
import * as Hex from './Hex.js';
/** Re-export of noble/curves X25519 utilities. */
export const noble = x25519;
/**
 * Creates a new X25519 key pair consisting of a private key and its corresponding public key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const { privateKey, publicKey } = X25519.createKeyPair()
 * ```
 *
 * @param options - The options to generate the key pair.
 * @returns The generated key pair containing both private and public keys.
 */
export function createKeyPair(options = {}) {
    const { as = 'Hex' } = options;
    const privateKey = randomPrivateKey({ as });
    const publicKey = getPublicKey({ privateKey, as });
    return {
        privateKey: privateKey,
        publicKey: publicKey,
    };
}
/**
 * Computes the X25519 public key from a provided private key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const publicKey = X25519.getPublicKey({ privateKey: '0x...' })
 * ```
 *
 * @param options - The options to compute the public key.
 * @returns The computed public key.
 */
export function getPublicKey(options) {
    const { as = 'Hex', privateKey } = options;
    const privateKeyBytes = Bytes.from(privateKey);
    const publicKeyBytes = x25519.getPublicKey(privateKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(publicKeyBytes);
    return publicKeyBytes;
}
/**
 * Computes a shared secret using X25519 elliptic curve Diffie-Hellman between a private key and a public key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const { privateKey: privateKeyA } = X25519.createKeyPair()
 * const { publicKey: publicKeyB } = X25519.createKeyPair()
 *
 * const sharedSecret = X25519.getSharedSecret({
 *   privateKey: privateKeyA,
 *   publicKey: publicKeyB
 * })
 * ```
 *
 * @param options - The options to compute the shared secret.
 * @returns The computed shared secret.
 */
export function getSharedSecret(options) {
    const { as = 'Hex', privateKey, publicKey } = options;
    const privateKeyBytes = Bytes.from(privateKey);
    const publicKeyBytes = Bytes.from(publicKey);
    const sharedSecretBytes = x25519.getSharedSecret(privateKeyBytes, publicKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(sharedSecretBytes);
    return sharedSecretBytes;
}
/**
 * Generates a random X25519 private key.
 *
 * @example
 * ```ts twoslash
 * import { X25519 } from 'ox'
 *
 * const privateKey = X25519.randomPrivateKey()
 * ```
 *
 * @param options - The options to generate the private key.
 * @returns The generated private key.
 */
export function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = x25519.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
//# sourceMappingURL=X25519.js.map