import { ed25519 } from '@noble/curves/ed25519';
import * as Bytes from './Bytes.js';
import * as Hex from './Hex.js';
/** Re-export of noble/curves Ed25519 utilities. */
export const noble = ed25519;
/**
 * Creates a new Ed25519 key pair consisting of a private key and its corresponding public key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const { privateKey, publicKey } = Ed25519.createKeyPair()
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
 * Computes the Ed25519 public key from a provided private key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const publicKey = Ed25519.getPublicKey({ privateKey: '0x...' })
 * ```
 *
 * @param options - The options to compute the public key.
 * @returns The computed public key.
 */
export function getPublicKey(options) {
    const { as = 'Hex', privateKey } = options;
    const privateKeyBytes = Bytes.from(privateKey);
    const publicKeyBytes = ed25519.getPublicKey(privateKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(publicKeyBytes);
    return publicKeyBytes;
}
/**
 * Generates a random Ed25519 private key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const privateKey = Ed25519.randomPrivateKey()
 * ```
 *
 * @param options - The options to generate the private key.
 * @returns The generated private key.
 */
export function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = ed25519.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
/**
 * Signs the payload with the provided private key and returns an Ed25519 signature.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const signature = Ed25519.sign({ // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   privateKey: '0x...' // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The signing options.
 * @returns The Ed25519 signature.
 */
export function sign(options) {
    const { as = 'Hex', payload, privateKey } = options;
    const payloadBytes = Bytes.from(payload);
    const privateKeyBytes = Bytes.from(privateKey);
    const signatureBytes = ed25519.sign(payloadBytes, privateKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(signatureBytes);
    return signatureBytes;
}
/**
 * Verifies a payload was signed by the provided public key.
 *
 * @example
 * ```ts twoslash
 * import { Ed25519 } from 'ox'
 *
 * const { privateKey, publicKey } = Ed25519.createKeyPair()
 * const signature = Ed25519.sign({ payload: '0xdeadbeef', privateKey })
 *
 * const verified = Ed25519.verify({ // [!code focus]
 *   publicKey, // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   signature, // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The verification options.
 * @returns Whether the payload was signed by the provided public key.
 */
export function verify(options) {
    const { payload, publicKey, signature } = options;
    const payloadBytes = Bytes.from(payload);
    const publicKeyBytes = Bytes.from(publicKey);
    const signatureBytes = Bytes.from(signature);
    return ed25519.verify(signatureBytes, payloadBytes, publicKeyBytes);
}
//# sourceMappingURL=Ed25519.js.map