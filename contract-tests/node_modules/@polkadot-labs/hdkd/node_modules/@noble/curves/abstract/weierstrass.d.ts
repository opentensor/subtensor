import { type CHash } from '../utils.ts';
import { type AffinePoint, type CurveLengths, type CurvePoint, type CurvePointCons } from './curve.ts';
import { type IField } from './modular.ts';
export type { AffinePoint };
type EndoBasis = [[bigint, bigint], [bigint, bigint]];
/**
 * When Weierstrass curve has `a=0`, it becomes Koblitz curve.
 * Koblitz curves allow using **efficiently-computable GLV endomorphism ψ**.
 * Endomorphism uses 2x less RAM, speeds up precomputation by 2x and ECDH / key recovery by 20%.
 * For precomputed wNAF it trades off 1/2 init time & 1/3 ram for 20% perf hit.
 *
 * Endomorphism consists of beta, lambda and splitScalar:
 *
 * 1. GLV endomorphism ψ transforms a point: `P = (x, y) ↦ ψ(P) = (β·x mod p, y)`
 * 2. GLV scalar decomposition transforms a scalar: `k ≡ k₁ + k₂·λ (mod n)`
 * 3. Then these are combined: `k·P = k₁·P + k₂·ψ(P)`
 * 4. Two 128-bit point-by-scalar multiplications + one point addition is faster than
 *    one 256-bit multiplication.
 *
 * where
 * * beta: β ∈ Fₚ with β³ = 1, β ≠ 1
 * * lambda: λ ∈ Fₙ with λ³ = 1, λ ≠ 1
 * * splitScalar decomposes k ↦ k₁, k₂, by using reduced basis vectors.
 *   Gauss lattice reduction calculates them from initial basis vectors `(n, 0), (-λ, 0)`
 *
 * Check out `test/misc/endomorphism.js` and
 * [gist](https://gist.github.com/paulmillr/eb670806793e84df628a7c434a873066).
 */
export type EndomorphismOpts = {
    beta: bigint;
    basises?: EndoBasis;
    splitScalar?: (k: bigint) => {
        k1neg: boolean;
        k1: bigint;
        k2neg: boolean;
        k2: bigint;
    };
};
export type ScalarEndoParts = {
    k1neg: boolean;
    k1: bigint;
    k2neg: boolean;
    k2: bigint;
};
/**
 * Splits scalar for GLV endomorphism.
 */
export declare function _splitEndoScalar(k: bigint, basis: EndoBasis, n: bigint): ScalarEndoParts;
/**
 * Option to enable hedged signatures with improved security.
 *
 * * Randomly generated k is bad, because broken CSPRNG would leak private keys.
 * * Deterministic k (RFC6979) is better; but is suspectible to fault attacks.
 *
 * We allow using technique described in RFC6979 3.6: additional k', a.k.a. adding randomness
 * to deterministic sig. If CSPRNG is broken & randomness is weak, it would STILL be as secure
 * as ordinary sig without ExtraEntropy.
 *
 * * `true` means "fetch data, from CSPRNG, incorporate it into k generation"
 * * `false` means "disable extra entropy, use purely deterministic k"
 * * `Uint8Array` passed means "incorporate following data into k generation"
 *
 * https://paulmillr.com/posts/deterministic-signatures/
 */
export type ECDSAExtraEntropy = boolean | Uint8Array;
/**
 * - `compact` is the default format
 * - `recovered` is the same as compact, but with an extra byte indicating recovery byte
 * - `der` is ASN.1 DER encoding
 */
export type ECDSASignatureFormat = 'compact' | 'recovered' | 'der';
/**
 * - `prehash`: (default: true) indicates whether to do sha256(message).
 *   When a custom hash is used, it must be set to `false`.
 */
export type ECDSARecoverOpts = {
    prehash?: boolean;
};
/**
 * - `prehash`: (default: true) indicates whether to do sha256(message).
 *   When a custom hash is used, it must be set to `false`.
 * - `lowS`: (default: true) prohibits signatures which have (sig.s >= CURVE.n/2n).
 *   Compatible with BTC/ETH. Setting `lowS: false` allows to create malleable signatures,
 *   which is default openssl behavior.
 *   Non-malleable signatures can still be successfully verified in openssl.
 * - `format`: (default: 'compact') 'compact' or 'recovered' with recovery byte
 */
export type ECDSAVerifyOpts = {
    prehash?: boolean;
    lowS?: boolean;
    format?: ECDSASignatureFormat;
};
/**
 * - `prehash`: (default: true) indicates whether to do sha256(message).
 *   When a custom hash is used, it must be set to `false`.
 * - `lowS`: (default: true) prohibits signatures which have (sig.s >= CURVE.n/2n).
 *   Compatible with BTC/ETH. Setting `lowS: false` allows to create malleable signatures,
 *   which is default openssl behavior.
 *   Non-malleable signatures can still be successfully verified in openssl.
 * - `format`: (default: 'compact') 'compact' or 'recovered' with recovery byte
 * - `extraEntropy`: (default: false) creates sigs with increased security, see {@link ECDSAExtraEntropy}
 */
export type ECDSASignOpts = {
    prehash?: boolean;
    lowS?: boolean;
    format?: ECDSASignatureFormat;
    extraEntropy?: ECDSAExtraEntropy;
};
/** Instance methods for 3D XYZ projective points. */
export interface WeierstrassPoint<T> extends CurvePoint<T, WeierstrassPoint<T>> {
    /** projective X coordinate. Different from affine x. */
    readonly X: T;
    /** projective Y coordinate. Different from affine y. */
    readonly Y: T;
    /** projective z coordinate */
    readonly Z: T;
    /** affine x coordinate. Different from projective X. */
    get x(): T;
    /** affine y coordinate. Different from projective Y. */
    get y(): T;
    /** Encodes point using IEEE P1363 (DER) encoding. First byte is 2/3/4. Default = isCompressed. */
    toBytes(isCompressed?: boolean): Uint8Array;
    toHex(isCompressed?: boolean): string;
}
/** Static methods for 3D XYZ projective points. */
export interface WeierstrassPointCons<T> extends CurvePointCons<WeierstrassPoint<T>> {
    /** Does NOT validate if the point is valid. Use `.assertValidity()`. */
    new (X: T, Y: T, Z: T): WeierstrassPoint<T>;
    CURVE(): WeierstrassOpts<T>;
}
/**
 * Weierstrass curve options.
 *
 * * p: prime characteristic (order) of finite field, in which arithmetics is done
 * * n: order of prime subgroup a.k.a total amount of valid curve points
 * * h: cofactor, usually 1. h*n is group order; n is subgroup order
 * * a: formula param, must be in field of p
 * * b: formula param, must be in field of p
 * * Gx: x coordinate of generator point a.k.a. base point
 * * Gy: y coordinate of generator point
 */
export type WeierstrassOpts<T> = Readonly<{
    p: bigint;
    n: bigint;
    h: bigint;
    a: T;
    b: T;
    Gx: T;
    Gy: T;
}>;
export type WeierstrassExtraOpts<T> = Partial<{
    Fp: IField<T>;
    Fn: IField<bigint>;
    allowInfinityPoint: boolean;
    endo: EndomorphismOpts;
    isTorsionFree: (c: WeierstrassPointCons<T>, point: WeierstrassPoint<T>) => boolean;
    clearCofactor: (c: WeierstrassPointCons<T>, point: WeierstrassPoint<T>) => WeierstrassPoint<T>;
    fromBytes: (bytes: Uint8Array) => AffinePoint<T>;
    toBytes: (c: WeierstrassPointCons<T>, point: WeierstrassPoint<T>, isCompressed: boolean) => Uint8Array;
}>;
/**
 * Options for ECDSA signatures over a Weierstrass curve.
 *
 * * lowS: (default: true) whether produced / verified signatures occupy low half of ecdsaOpts.p. Prevents malleability.
 * * hmac: (default: noble-hashes hmac) function, would be used to init hmac-drbg for k generation.
 * * randomBytes: (default: webcrypto os-level CSPRNG) custom method for fetching secure randomness.
 * * bits2int, bits2int_modN: used in sigs, sometimes overridden by curves
 */
export type ECDSAOpts = Partial<{
    lowS: boolean;
    hmac: (key: Uint8Array, message: Uint8Array) => Uint8Array;
    randomBytes: (bytesLength?: number) => Uint8Array;
    bits2int: (bytes: Uint8Array) => bigint;
    bits2int_modN: (bytes: Uint8Array) => bigint;
}>;
/**
 * Elliptic Curve Diffie-Hellman interface.
 * Provides keygen, secret-to-public conversion, calculating shared secrets.
 */
export interface ECDH {
    keygen: (seed?: Uint8Array) => {
        secretKey: Uint8Array;
        publicKey: Uint8Array;
    };
    getPublicKey: (secretKey: Uint8Array, isCompressed?: boolean) => Uint8Array;
    getSharedSecret: (secretKeyA: Uint8Array, publicKeyB: Uint8Array, isCompressed?: boolean) => Uint8Array;
    Point: WeierstrassPointCons<bigint>;
    utils: {
        isValidSecretKey: (secretKey: Uint8Array) => boolean;
        isValidPublicKey: (publicKey: Uint8Array, isCompressed?: boolean) => boolean;
        randomSecretKey: (seed?: Uint8Array) => Uint8Array;
    };
    lengths: CurveLengths;
}
/**
 * ECDSA interface.
 * Only supported for prime fields, not Fp2 (extension fields).
 */
export interface ECDSA extends ECDH {
    sign: (message: Uint8Array, secretKey: Uint8Array, opts?: ECDSASignOpts) => Uint8Array;
    verify: (signature: Uint8Array, message: Uint8Array, publicKey: Uint8Array, opts?: ECDSAVerifyOpts) => boolean;
    recoverPublicKey(signature: Uint8Array, message: Uint8Array, opts?: ECDSARecoverOpts): Uint8Array;
    Signature: ECDSASignatureCons;
}
export declare class DERErr extends Error {
    constructor(m?: string);
}
export type IDER = {
    Err: typeof DERErr;
    _tlv: {
        encode: (tag: number, data: string) => string;
        decode(tag: number, data: Uint8Array): {
            v: Uint8Array;
            l: Uint8Array;
        };
    };
    _int: {
        encode(num: bigint): string;
        decode(data: Uint8Array): bigint;
    };
    toSig(hex: string | Uint8Array): {
        r: bigint;
        s: bigint;
    };
    hexFromSig(sig: {
        r: bigint;
        s: bigint;
    }): string;
};
/**
 * ASN.1 DER encoding utilities. ASN is very complex & fragile. Format:
 *
 *     [0x30 (SEQUENCE), bytelength, 0x02 (INTEGER), intLength, R, 0x02 (INTEGER), intLength, S]
 *
 * Docs: https://letsencrypt.org/docs/a-warm-welcome-to-asn1-and-der/, https://luca.ntop.org/Teaching/Appunti/asn1.html
 */
export declare const DER: IDER;
/**
 * Creates weierstrass Point constructor, based on specified curve options.
 *
 * See {@link WeierstrassOpts}.
 *
 * @example
```js
const opts = {
  p: 0xfffffffffffffffffffffffffffffffeffffac73n,
  n: 0x100000000000000000001b8fa16dfab9aca16b6b3n,
  h: 1n,
  a: 0n,
  b: 7n,
  Gx: 0x3b4c382ce37aa192a4019e763036f4f5dd4d7ebbn,
  Gy: 0x938cf935318fdced6bc28286531733c3f03c4feen,
};
const secp160k1_Point = weierstrass(opts);
```
 */
export declare function weierstrass<T>(params: WeierstrassOpts<T>, extraOpts?: WeierstrassExtraOpts<T>): WeierstrassPointCons<T>;
/** Methods of ECDSA signature instance. */
export interface ECDSASignature {
    readonly r: bigint;
    readonly s: bigint;
    readonly recovery?: number;
    addRecoveryBit(recovery: number): ECDSASignature & {
        readonly recovery: number;
    };
    hasHighS(): boolean;
    recoverPublicKey(messageHash: Uint8Array): WeierstrassPoint<bigint>;
    toBytes(format?: string): Uint8Array;
    toHex(format?: string): string;
}
/** Methods of ECDSA signature constructor. */
export type ECDSASignatureCons = {
    new (r: bigint, s: bigint, recovery?: number): ECDSASignature;
    fromBytes(bytes: Uint8Array, format?: ECDSASignatureFormat): ECDSASignature;
    fromHex(hex: string, format?: ECDSASignatureFormat): ECDSASignature;
};
/**
 * Implementation of the Shallue and van de Woestijne method for any weierstrass curve.
 * TODO: check if there is a way to merge this with uvRatio in Edwards; move to modular.
 * b = True and y = sqrt(u / v) if (u / v) is square in F, and
 * b = False and y = sqrt(Z * (u / v)) otherwise.
 * @param Fp
 * @param Z
 * @returns
 */
export declare function SWUFpSqrtRatio<T>(Fp: IField<T>, Z: T): (u: T, v: T) => {
    isValid: boolean;
    value: T;
};
/**
 * Simplified Shallue-van de Woestijne-Ulas Method
 * https://www.rfc-editor.org/rfc/rfc9380#section-6.6.2
 */
export declare function mapToCurveSimpleSWU<T>(Fp: IField<T>, opts: {
    A: T;
    B: T;
    Z: T;
}): (u: T) => {
    x: T;
    y: T;
};
/**
 * Sometimes users only need getPublicKey, getSharedSecret, and secret key handling.
 * This helper ensures no signature functionality is present. Less code, smaller bundle size.
 */
export declare function ecdh(Point: WeierstrassPointCons<bigint>, ecdhOpts?: {
    randomBytes?: (bytesLength?: number) => Uint8Array;
}): ECDH;
/**
 * Creates ECDSA signing interface for given elliptic curve `Point` and `hash` function.
 *
 * @param Point created using {@link weierstrass} function
 * @param hash used for 1) message prehash-ing 2) k generation in `sign`, using hmac_drbg(hash)
 * @param ecdsaOpts rarely needed, see {@link ECDSAOpts}
 *
 * @example
 * ```js
 * const p256_Point = weierstrass(...);
 * const p256_sha256 = ecdsa(p256_Point, sha256);
 * const p256_sha224 = ecdsa(p256_Point, sha224);
 * const p256_sha224_r = ecdsa(p256_Point, sha224, { randomBytes: (length) => { ... } });
 * ```
 */
export declare function ecdsa(Point: WeierstrassPointCons<bigint>, hash: CHash, ecdsaOpts?: ECDSAOpts): ECDSA;
//# sourceMappingURL=weierstrass.d.ts.map