import { secp256r1 } from '@noble/curves/p256';
import * as Bytes from './Bytes.js';
import * as Hex from './Hex.js';
import * as Entropy from './internal/entropy.js';
import * as PublicKey from './PublicKey.js';
/** Re-export of noble/curves P256 utilities. */
export const noble = secp256r1;
/**
 * Creates a new P256 ECDSA key pair consisting of a private key and its corresponding public key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const { privateKey, publicKey } = P256.createKeyPair()
 * ```
 *
 * @param options - The options to generate the key pair.
 * @returns The generated key pair containing both private and public keys.
 */
export function createKeyPair(options = {}) {
    const { as = 'Hex' } = options;
    const privateKey = randomPrivateKey({ as });
    const publicKey = getPublicKey({ privateKey });
    return {
        privateKey: privateKey,
        publicKey,
    };
}
/**
 * Computes the P256 ECDSA public key from a provided private key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const publicKey = P256.getPublicKey({ privateKey: '0x...' })
 * ```
 *
 * @param options - The options to compute the public key.
 * @returns The computed public key.
 */
export function getPublicKey(options) {
    const { privateKey } = options;
    const point = secp256r1.ProjectivePoint.fromPrivateKey(typeof privateKey === 'string'
        ? privateKey.slice(2)
        : Hex.fromBytes(privateKey).slice(2));
    return PublicKey.from(point);
}
/**
 * Computes a shared secret using ECDH (Elliptic Curve Diffie-Hellman) between a private key and a public key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const { privateKey: privateKeyA } = P256.createKeyPair()
 * const { publicKey: publicKeyB } = P256.createKeyPair()
 *
 * const sharedSecret = P256.getSharedSecret({
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
    const point = secp256r1.ProjectivePoint.fromHex(PublicKey.toHex(publicKey).slice(2));
    const privateKeyHex = typeof privateKey === 'string'
        ? privateKey.slice(2)
        : Hex.fromBytes(privateKey).slice(2);
    const sharedPoint = point.multiply(secp256r1.utils.normPrivateKeyToScalar(privateKeyHex));
    const sharedSecret = sharedPoint.toRawBytes(true); // compressed format
    if (as === 'Hex')
        return Hex.fromBytes(sharedSecret);
    return sharedSecret;
}
/**
 * Generates a random P256 ECDSA private key.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const privateKey = P256.randomPrivateKey()
 * ```
 *
 * @param options - The options to generate the private key.
 * @returns The generated private key.
 */
export function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = secp256r1.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
/**
 * Recovers the signing public key from the signed payload and signature.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const signature = P256.sign({ payload: '0xdeadbeef', privateKey: '0x...' })
 *
 * const publicKey = P256.recoverPublicKey({ // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   signature, // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The recovery options.
 * @returns The recovered public key.
 */
export function recoverPublicKey(options) {
    const { payload, signature } = options;
    const { r, s, yParity } = signature;
    const signature_ = new secp256r1.Signature(BigInt(r), BigInt(s)).addRecoveryBit(yParity);
    const payload_ = payload instanceof Uint8Array ? Hex.fromBytes(payload) : payload;
    const point = signature_.recoverPublicKey(payload_.substring(2));
    return PublicKey.from(point);
}
/**
 * Signs the payload with the provided private key and returns a P256 signature.
 *
 * @example
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const signature = P256.sign({ // [!code focus]
 *   payload: '0xdeadbeef', // [!code focus]
 *   privateKey: '0x...' // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @param options - The signing options.
 * @returns The ECDSA {@link ox#Signature.Signature}.
 */
export function sign(options) {
    const { extraEntropy = Entropy.extraEntropy, hash, payload, privateKey, } = options;
    const { r, s, recovery } = secp256r1.sign(payload instanceof Uint8Array ? payload : Bytes.fromHex(payload), privateKey instanceof Uint8Array ? privateKey : Bytes.fromHex(privateKey), {
        extraEntropy: typeof extraEntropy === 'boolean'
            ? extraEntropy
            : Hex.from(extraEntropy).slice(2),
        lowS: true,
        ...(hash ? { prehash: true } : {}),
    });
    return {
        r,
        s,
        yParity: recovery,
    };
}
/**
 * Verifies a payload was signed by the provided public key.
 *
 * @example
 *
 * ```ts twoslash
 * import { P256 } from 'ox'
 *
 * const { privateKey, publicKey } = P256.createKeyPair()
 * const signature = P256.sign({ payload: '0xdeadbeef', privateKey })
 *
 * const verified = P256.verify({ // [!code focus]
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
    const { hash, payload, publicKey, signature } = options;
    return secp256r1.verify(signature, payload instanceof Uint8Array ? payload : Bytes.fromHex(payload), PublicKey.toHex(publicKey).substring(2), ...(hash ? [{ prehash: true, lowS: true }] : []));
}
//# sourceMappingURL=P256.js.map