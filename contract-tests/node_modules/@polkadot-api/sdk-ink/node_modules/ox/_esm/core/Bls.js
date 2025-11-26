import { bls12_381 as bls } from '@noble/curves/bls12-381';
import * as Bytes from './Bytes.js';
import * as Hex from './Hex.js';
/** Re-export of noble/curves BLS12-381 utilities. */
export const noble = bls;
// eslint-disable-next-line jsdoc/require-jsdoc
export function aggregate(points) {
    const group = typeof points[0]?.x === 'bigint' ? bls.G1 : bls.G2;
    const point = points.reduce((acc, point) => acc.add(new group.ProjectivePoint(point.x, point.y, point.z)), group.ProjectivePoint.ZERO);
    return {
        x: point.px,
        y: point.py,
        z: point.pz,
    };
}
/**
 * Creates a new BLS12-381 key pair consisting of a private key and its corresponding public key.
 *
 * - G1 Point (Default):
 *   - short (48 bytes)
 *   - computes longer G2 Signatures (96 bytes)
 * - G2 Point:
 *   - long (96 bytes)
 *   - computes short G1 Signatures (48 bytes)
 *
 * @example
 * ### Short G1 Public Keys (Default)
 *
 * ```ts twoslash
 * import { Bls } from 'ox'
 *
 * const { publicKey } = Bls.createKeyPair()
 * //      ^?
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 * @example
 * ### Long G2 Public Keys
 *
 * A G2 Public Key can be derived as a G2 point (96 bytes) using `size: 'long-key:short-sig'`.
 *
 * This will allow you to compute G1 Signatures (48 bytes) with {@link ox#Bls.(sign:function)}.
 *
 * ```ts twoslash
 * import { Bls } from 'ox'
 *
 * const { publicKey } = Bls.createKeyPair({
 *   size: 'long-key:short-sig',
 * })
 *
 * publicKey
 * // ^?
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 * ### Serializing
 *
 * Public Keys can be serialized to hex or bytes using {@link ox#BlsPoint.(toHex:function)} or {@link ox#BlsPoint.(toBytes:function)}:
 *
 * ```ts twoslash
 * import { Bls, BlsPoint } from 'ox'
 *
 * const { publicKey } = Bls.createKeyPair()
 *
 * const publicKeyHex = BlsPoint.toHex(publicKey)
 * //    ^?
 *
 *
 * const publicKeyBytes = BlsPoint.toBytes(publicKey)
 * //    ^?
 *
 * ```
 *
 * They can also be deserialized from hex or bytes using {@link ox#BlsPoint.(fromHex:function)} or {@link ox#BlsPoint.(fromBytes:function)}:
 *
 * ```ts twoslash
 * import { Bls, BlsPoint } from 'ox'
 *
 * const publicKeyHex = '0x...'
 *
 * const publicKey = BlsPoint.fromHex(publicKeyHex, 'G1')
 * //    ^?
 *
 *
 *
 *
 *
 *
 *
 * ```
 *
 * @param options - The options to generate the key pair.
 * @returns The generated key pair containing both private and public keys.
 */
export function createKeyPair(options = {}) {
    const { as = 'Hex', size = 'short-key:long-sig' } = options;
    const privateKey = randomPrivateKey({ as });
    const publicKey = getPublicKey({ privateKey, size });
    return {
        privateKey: privateKey,
        publicKey: publicKey,
    };
}
// eslint-disable-next-line jsdoc/require-jsdoc
export function getPublicKey(options) {
    const { privateKey, size = 'short-key:long-sig' } = options;
    const group = size === 'short-key:long-sig' ? bls.G1 : bls.G2;
    const { px, py, pz } = group.ProjectivePoint.fromPrivateKey(Hex.from(privateKey).slice(2));
    return { x: px, y: py, z: pz };
}
/**
 * Generates a random BLS12-381 private key.
 *
 * @example
 * ```ts twoslash
 * import { Bls } from 'ox'
 *
 * const privateKey = Bls.randomPrivateKey()
 * ```
 *
 * @param options - The options to generate the private key.
 * @returns The generated private key.
 */
export function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = bls.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
// eslint-disable-next-line jsdoc/require-jsdoc
export function sign(options) {
    const { payload, privateKey, suite, size = 'short-key:long-sig' } = options;
    const payloadGroup = size === 'short-key:long-sig' ? bls.G2 : bls.G1;
    const payloadPoint = payloadGroup.hashToCurve(Bytes.from(payload), suite ? { DST: Bytes.fromString(suite) } : undefined);
    const privateKeyGroup = size === 'short-key:long-sig' ? bls.G1 : bls.G2;
    const signature = payloadPoint.multiply(privateKeyGroup.normPrivateKeyToScalar(privateKey.slice(2)));
    return {
        x: signature.px,
        y: signature.py,
        z: signature.pz,
    };
}
/**
 * Verifies a payload was signed by the provided public key(s).
 *
 * @example
 *
 * ```ts twoslash
 * import { Bls, Hex } from 'ox'
 *
 * const payload = Hex.random(32)
 * const privateKey = Bls.randomPrivateKey()
 *
 * const publicKey = Bls.getPublicKey({ privateKey })
 * const signature = Bls.sign({ payload, privateKey })
 *
 * const verified = Bls.verify({ // [!code focus]
 *   payload, // [!code focus]
 *   publicKey, // [!code focus]
 *   signature, // [!code focus]
 * }) // [!code focus]
 * ```
 *
 * @example
 * ### Verify Aggregated Signatures
 *
 * We can also pass a public key and signature that was aggregated with {@link ox#Bls.(aggregate:function)} to `Bls.verify`.
 *
 * ```ts twoslash
 * import { Bls, Hex } from 'ox'
 *
 * const payload = Hex.random(32)
 * const privateKeys = Array.from({ length: 100 }, () => Bls.randomPrivateKey())
 *
 * const publicKeys = privateKeys.map((privateKey) =>
 *   Bls.getPublicKey({ privateKey }),
 * )
 * const signatures = privateKeys.map((privateKey) =>
 *   Bls.sign({ payload, privateKey }),
 * )
 *
 * const publicKey = Bls.aggregate(publicKeys) // [!code focus]
 * const signature = Bls.aggregate(signatures) // [!code focus]
 *
 * const valid = Bls.verify({ payload, publicKey, signature }) // [!code focus]
 * ```
 *
 * @param options - Verification options.
 * @returns Whether the payload was signed by the provided public key.
 */
export function verify(options) {
    const { payload, suite } = options;
    const publicKey = options.publicKey;
    const signature = options.signature;
    const isShortSig = typeof signature.x === 'bigint';
    const group = isShortSig ? bls.G1 : bls.G2;
    const payloadPoint = group.hashToCurve(Bytes.from(payload), suite ? { DST: Bytes.fromString(suite) } : undefined);
    const shortSigPairing = () => bls.pairingBatch([
        {
            g1: payloadPoint,
            g2: new bls.G2.ProjectivePoint(publicKey.x, publicKey.y, publicKey.z),
        },
        {
            g1: new bls.G1.ProjectivePoint(signature.x, signature.y, signature.z),
            g2: bls.G2.ProjectivePoint.BASE.negate(),
        },
    ]);
    const longSigPairing = () => bls.pairingBatch([
        {
            g1: new bls.G1.ProjectivePoint(publicKey.x, publicKey.y, publicKey.z).negate(),
            g2: payloadPoint,
        },
        {
            g1: bls.G1.ProjectivePoint.BASE,
            g2: new bls.G2.ProjectivePoint(signature.x, signature.y, signature.z),
        },
    ]);
    return bls.fields.Fp12.eql(isShortSig ? shortSigPairing() : longSigPairing(), bls.fields.Fp12.ONE);
}
//# sourceMappingURL=Bls.js.map