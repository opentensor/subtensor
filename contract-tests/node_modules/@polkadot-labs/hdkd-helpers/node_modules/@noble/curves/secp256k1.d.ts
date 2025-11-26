import { type CurveLengths } from './abstract/curve.ts';
import { type H2CHasher } from './abstract/hash-to-curve.ts';
import { type ECDSA, type WeierstrassPoint as PointType, type WeierstrassPointCons } from './abstract/weierstrass.ts';
/**
 * secp256k1 curve: ECDSA and ECDH methods.
 *
 * Uses sha256 to hash messages. To use a different hash,
 * pass `{ prehash: false }` to sign / verify.
 *
 * @example
 * ```js
 * import { secp256k1 } from '@noble/curves/secp256k1.js';
 * const { secretKey, publicKey } = secp256k1.keygen();
 * // const publicKey = secp256k1.getPublicKey(secretKey);
 * const msg = new TextEncoder().encode('hello noble');
 * const sig = secp256k1.sign(msg, secretKey);
 * const isValid = secp256k1.verify(sig, msg, publicKey);
 * // const sigKeccak = secp256k1.sign(keccak256(msg), secretKey, { prehash: false });
 * ```
 */
export declare const secp256k1: ECDSA;
declare function taggedHash(tag: string, ...messages: Uint8Array[]): Uint8Array;
/**
 * lift_x from BIP340. Convert 32-byte x coordinate to elliptic curve point.
 * @returns valid point checked for being on-curve
 */
declare function lift_x(x: bigint): PointType<bigint>;
/**
 * Schnorr public key is just `x` coordinate of Point as per BIP340.
 */
declare function schnorrGetPublicKey(secretKey: Uint8Array): Uint8Array;
/**
 * Creates Schnorr signature as per BIP340. Verifies itself before returning anything.
 * auxRand is optional and is not the sole source of k generation: bad CSPRNG won't be dangerous.
 */
declare function schnorrSign(message: Uint8Array, secretKey: Uint8Array, auxRand?: Uint8Array): Uint8Array;
/**
 * Verifies Schnorr signature.
 * Will swallow errors & return false except for initial type validation of arguments.
 */
declare function schnorrVerify(signature: Uint8Array, message: Uint8Array, publicKey: Uint8Array): boolean;
export type SecpSchnorr = {
    keygen: (seed?: Uint8Array) => {
        secretKey: Uint8Array;
        publicKey: Uint8Array;
    };
    getPublicKey: typeof schnorrGetPublicKey;
    sign: typeof schnorrSign;
    verify: typeof schnorrVerify;
    Point: WeierstrassPointCons<bigint>;
    utils: {
        randomSecretKey: (seed?: Uint8Array) => Uint8Array;
        pointToBytes: (point: PointType<bigint>) => Uint8Array;
        lift_x: typeof lift_x;
        taggedHash: typeof taggedHash;
    };
    lengths: CurveLengths;
};
/**
 * Schnorr signatures over secp256k1.
 * https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki
 * @example
 * ```js
 * import { schnorr } from '@noble/curves/secp256k1.js';
 * const { secretKey, publicKey } = schnorr.keygen();
 * // const publicKey = schnorr.getPublicKey(secretKey);
 * const msg = new TextEncoder().encode('hello');
 * const sig = schnorr.sign(msg, secretKey);
 * const isValid = schnorr.verify(sig, msg, publicKey);
 * ```
 */
export declare const schnorr: SecpSchnorr;
/** Hashing / encoding to secp256k1 points / field. RFC 9380 methods. */
export declare const secp256k1_hasher: H2CHasher<WeierstrassPointCons<bigint>>;
export {};
//# sourceMappingURL=secp256k1.d.ts.map