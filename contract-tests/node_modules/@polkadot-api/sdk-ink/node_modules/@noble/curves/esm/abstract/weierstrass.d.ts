/**
 * Short Weierstrass curve methods. The formula is: y² = x³ + ax + b.
 *
 * ### Parameters
 *
 * To initialize a weierstrass curve, one needs to pass following params:
 *
 * * a: formula param
 * * b: formula param
 * * Fp: finite field of prime characteristic P; may be complex (Fp2). Arithmetics is done in field
 * * n: order of prime subgroup a.k.a total amount of valid curve points
 * * Gx: Base point (x, y) aka generator point. Gx = x coordinate
 * * Gy: ...y coordinate
 * * h: cofactor, usually 1. h*n = curve group order (n is only subgroup order)
 * * lowS: whether to enable (default) or disable "low-s" non-malleable signatures
 *
 * ### Design rationale for types
 *
 * * Interaction between classes from different curves should fail:
 *   `k256.Point.BASE.add(p256.Point.BASE)`
 * * For this purpose we want to use `instanceof` operator, which is fast and works during runtime
 * * Different calls of `curve()` would return different classes -
 *   `curve(params) !== curve(params)`: if somebody decided to monkey-patch their curve,
 *   it won't affect others
 *
 * TypeScript can't infer types for classes created inside a function. Classes is one instance
 * of nominative types in TypeScript and interfaces only check for shape, so it's hard to create
 * unique type for every function call.
 *
 * We can use generic types via some param, like curve opts, but that would:
 *     1. Enable interaction between `curve(params)` and `curve(params)` (curves of same params)
 *     which is hard to debug.
 *     2. Params can be generic and we can't enforce them to be constant value:
 *     if somebody creates curve from non-constant params,
 *     it would be allowed to interact with other curves with non-constant params
 *
 * @todo https://www.typescriptlang.org/docs/handbook/release-notes/typescript-2-7.html#unique-symbol
 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import { type AffinePoint, type BasicCurve, type Group, type GroupConstructor } from './curve.ts';
import { type IField } from './modular.ts';
import { type CHash, type Hex, type PrivKey } from './utils.ts';
export type { AffinePoint };
type HmacFnSync = (key: Uint8Array, ...messages: Uint8Array[]) => Uint8Array;
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
    splitScalar: (k: bigint) => {
        k1neg: boolean;
        k1: bigint;
        k2neg: boolean;
        k2: bigint;
    };
};
export type BasicWCurve<T> = BasicCurve<T> & {
    a: T;
    b: T;
    allowedPrivateKeyLengths?: readonly number[];
    wrapPrivateKey?: boolean;
    endo?: EndomorphismOpts;
    isTorsionFree?: (c: ProjConstructor<T>, point: ProjPointType<T>) => boolean;
    clearCofactor?: (c: ProjConstructor<T>, point: ProjPointType<T>) => ProjPointType<T>;
};
export type Entropy = Hex | boolean;
export type SignOpts = {
    lowS?: boolean;
    extraEntropy?: Entropy;
    prehash?: boolean;
};
export type VerOpts = {
    lowS?: boolean;
    prehash?: boolean;
    format?: 'compact' | 'der' | undefined;
};
export interface ProjPointType<T> extends Group<ProjPointType<T>> {
    readonly px: T;
    readonly py: T;
    readonly pz: T;
    get x(): T;
    get y(): T;
    toAffine(iz?: T): AffinePoint<T>;
    toHex(isCompressed?: boolean): string;
    toRawBytes(isCompressed?: boolean): Uint8Array;
    assertValidity(): void;
    hasEvenY(): boolean;
    multiplyUnsafe(scalar: bigint): ProjPointType<T>;
    multiplyAndAddUnsafe(Q: ProjPointType<T>, a: bigint, b: bigint): ProjPointType<T> | undefined;
    isTorsionFree(): boolean;
    clearCofactor(): ProjPointType<T>;
    _setWindowSize(windowSize: number): void;
}
export interface ProjConstructor<T> extends GroupConstructor<ProjPointType<T>> {
    new (x: T, y: T, z: T): ProjPointType<T>;
    fromAffine(p: AffinePoint<T>): ProjPointType<T>;
    fromHex(hex: Hex): ProjPointType<T>;
    fromPrivateKey(privateKey: PrivKey): ProjPointType<T>;
    normalizeZ(points: ProjPointType<T>[]): ProjPointType<T>[];
    msm(points: ProjPointType<T>[], scalars: bigint[]): ProjPointType<T>;
}
export type CurvePointsType<T> = BasicWCurve<T> & {
    fromBytes?: (bytes: Uint8Array) => AffinePoint<T>;
    toBytes?: (c: ProjConstructor<T>, point: ProjPointType<T>, isCompressed: boolean) => Uint8Array;
};
export type CurvePointsTypeWithLength<T> = Readonly<CurvePointsType<T> & {
    nByteLength: number;
    nBitLength: number;
}>;
declare function validatePointOpts<T>(curve: CurvePointsType<T>): CurvePointsTypeWithLength<T>;
export type CurvePointsRes<T> = {
    CURVE: ReturnType<typeof validatePointOpts<T>>;
    ProjectivePoint: ProjConstructor<T>;
    normPrivateKeyToScalar: (key: PrivKey) => bigint;
    weierstrassEquation: (x: T) => T;
    isWithinCurveOrder: (num: bigint) => boolean;
};
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
export declare function weierstrassPoints<T>(opts: CurvePointsType<T>): CurvePointsRes<T>;
export interface SignatureType {
    readonly r: bigint;
    readonly s: bigint;
    readonly recovery?: number;
    assertValidity(): void;
    addRecoveryBit(recovery: number): RecoveredSignatureType;
    hasHighS(): boolean;
    normalizeS(): SignatureType;
    recoverPublicKey(msgHash: Hex): ProjPointType<bigint>;
    toCompactRawBytes(): Uint8Array;
    toCompactHex(): string;
    toDERRawBytes(isCompressed?: boolean): Uint8Array;
    toDERHex(isCompressed?: boolean): string;
}
export type RecoveredSignatureType = SignatureType & {
    readonly recovery: number;
};
export type SignatureConstructor = {
    new (r: bigint, s: bigint): SignatureType;
    fromCompact(hex: Hex): SignatureType;
    fromDER(hex: Hex): SignatureType;
};
type SignatureLike = {
    r: bigint;
    s: bigint;
};
export type PubKey = Hex | ProjPointType<bigint>;
export type CurveType = BasicWCurve<bigint> & {
    hash: CHash;
    hmac: HmacFnSync;
    randomBytes: (bytesLength?: number) => Uint8Array;
    lowS?: boolean;
    bits2int?: (bytes: Uint8Array) => bigint;
    bits2int_modN?: (bytes: Uint8Array) => bigint;
};
declare function validateOpts(curve: CurveType): Readonly<CurveType & {
    nByteLength: number;
    nBitLength: number;
}>;
export type CurveFn = {
    CURVE: ReturnType<typeof validateOpts>;
    getPublicKey: (privateKey: PrivKey, isCompressed?: boolean) => Uint8Array;
    getSharedSecret: (privateA: PrivKey, publicB: Hex, isCompressed?: boolean) => Uint8Array;
    sign: (msgHash: Hex, privKey: PrivKey, opts?: SignOpts) => RecoveredSignatureType;
    verify: (signature: Hex | SignatureLike, msgHash: Hex, publicKey: Hex, opts?: VerOpts) => boolean;
    ProjectivePoint: ProjConstructor<bigint>;
    Signature: SignatureConstructor;
    utils: {
        normPrivateKeyToScalar: (key: PrivKey) => bigint;
        isValidPrivateKey(privateKey: PrivKey): boolean;
        randomPrivateKey: () => Uint8Array;
        precompute: (windowSize?: number, point?: ProjPointType<bigint>) => ProjPointType<bigint>;
    };
};
/**
 * Creates short weierstrass curve and ECDSA signature methods for it.
 * @example
 * import { Field } from '@noble/curves/abstract/modular';
 * // Before that, define BigInt-s: a, b, p, n, Gx, Gy
 * const curve = weierstrass({ a, b, Fp: Field(p), n, Gx, Gy, h: 1n })
 */
export declare function weierstrass(curveDef: CurveType): CurveFn;
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
//# sourceMappingURL=weierstrass.d.ts.map