import { type CurveLengths } from './curve.ts';
import { type H2CHasher, type H2CHashOpts, type H2COpts, type MapToCurve } from './hash-to-curve.ts';
import { type IField } from './modular.ts';
import type { Fp12, Fp12Bls, Fp2, Fp2Bls, Fp6Bls } from './tower.ts';
import { type WeierstrassPoint, type WeierstrassPointCons } from './weierstrass.ts';
type Fp = bigint;
export type BlsTwistType = 'multiplicative' | 'divisive';
export type BlsShortSignatureCoder<Fp> = {
    fromBytes(bytes: Uint8Array): WeierstrassPoint<Fp>;
    fromHex(hex: string): WeierstrassPoint<Fp>;
    toBytes(point: WeierstrassPoint<Fp>): Uint8Array;
    toHex(point: WeierstrassPoint<Fp>): string;
};
export type BlsLongSignatureCoder<Fp> = {
    fromBytes(bytes: Uint8Array): WeierstrassPoint<Fp>;
    fromHex(hex: string): WeierstrassPoint<Fp>;
    toBytes(point: WeierstrassPoint<Fp>): Uint8Array;
    toHex(point: WeierstrassPoint<Fp>): string;
};
export type BlsFields = {
    Fp: IField<Fp>;
    Fr: IField<bigint>;
    Fp2: Fp2Bls;
    Fp6: Fp6Bls;
    Fp12: Fp12Bls;
};
export type BlsPostPrecomputePointAddFn = (Rx: Fp2, Ry: Fp2, Rz: Fp2, Qx: Fp2, Qy: Fp2) => {
    Rx: Fp2;
    Ry: Fp2;
    Rz: Fp2;
};
export type BlsPostPrecomputeFn = (Rx: Fp2, Ry: Fp2, Rz: Fp2, Qx: Fp2, Qy: Fp2, pointAdd: BlsPostPrecomputePointAddFn) => void;
export type BlsPairing = {
    lengths: CurveLengths;
    Fr: IField<bigint>;
    Fp12: Fp12Bls;
    calcPairingPrecomputes: (p: WeierstrassPoint<Fp2>) => Precompute;
    millerLoopBatch: (pairs: [Precompute, Fp, Fp][]) => Fp12;
    pairing: (P: WeierstrassPoint<Fp>, Q: WeierstrassPoint<Fp2>, withFinalExponent?: boolean) => Fp12;
    pairingBatch: (pairs: {
        g1: WeierstrassPoint<Fp>;
        g2: WeierstrassPoint<Fp2>;
    }[], withFinalExponent?: boolean) => Fp12;
    randomSecretKey: (seed?: Uint8Array) => Uint8Array;
};
export type BlsPairingParams = {
    ateLoopSize: bigint;
    xNegative: boolean;
    twistType: BlsTwistType;
    randomBytes?: (len?: number) => Uint8Array;
    postPrecompute?: BlsPostPrecomputeFn;
};
export type BlsHasherParams = {
    mapToG1?: MapToCurve<Fp>;
    mapToG2?: MapToCurve<Fp2>;
    hasherOpts: H2COpts;
    hasherOptsG1: H2COpts;
    hasherOptsG2: H2COpts;
};
type PrecomputeSingle = [Fp2, Fp2, Fp2][];
type Precompute = PrecomputeSingle[];
/**
 * BLS consists of two curves: G1 and G2:
 * - G1 is a subgroup of (x, y) E(Fq) over y² = x³ + 4.
 * - G2 is a subgroup of ((x₁, x₂+i), (y₁, y₂+i)) E(Fq²) over y² = x³ + 4(1 + i) where i is √-1
 */
export interface BlsCurvePair {
    lengths: CurveLengths;
    millerLoopBatch: BlsPairing['millerLoopBatch'];
    pairing: BlsPairing['pairing'];
    pairingBatch: BlsPairing['pairingBatch'];
    G1: {
        Point: WeierstrassPointCons<Fp>;
    };
    G2: {
        Point: WeierstrassPointCons<Fp2>;
    };
    fields: {
        Fp: IField<Fp>;
        Fp2: Fp2Bls;
        Fp6: Fp6Bls;
        Fp12: Fp12Bls;
        Fr: IField<bigint>;
    };
    utils: {
        randomSecretKey: (seed?: Uint8Array) => Uint8Array;
        calcPairingPrecomputes: BlsPairing['calcPairingPrecomputes'];
    };
    params: {
        ateLoopSize: bigint;
        twistType: BlsTwistType;
    };
}
export interface BlsCurvePairWithHashers extends BlsCurvePair {
    G1: H2CHasher<WeierstrassPointCons<Fp>>;
    G2: H2CHasher<WeierstrassPointCons<Fp2>>;
}
export interface BlsCurvePairWithSignatures extends BlsCurvePairWithHashers {
    longSignatures: BlsSigs<bigint, Fp2>;
    shortSignatures: BlsSigs<Fp2, bigint>;
}
type BLSInput = Uint8Array;
export interface BlsSigs<P, S> {
    lengths: CurveLengths;
    keygen(seed?: Uint8Array): {
        secretKey: Uint8Array;
        publicKey: WeierstrassPoint<P>;
    };
    getPublicKey(secretKey: Uint8Array): WeierstrassPoint<P>;
    sign(hashedMessage: WeierstrassPoint<S>, secretKey: Uint8Array): WeierstrassPoint<S>;
    verify(signature: WeierstrassPoint<S> | BLSInput, message: WeierstrassPoint<S>, publicKey: WeierstrassPoint<P> | BLSInput): boolean;
    verifyBatch: (signature: WeierstrassPoint<S> | BLSInput, items: {
        message: WeierstrassPoint<S>;
        publicKey: WeierstrassPoint<P> | BLSInput;
    }[]) => boolean;
    aggregatePublicKeys(publicKeys: (WeierstrassPoint<P> | BLSInput)[]): WeierstrassPoint<P>;
    aggregateSignatures(signatures: (WeierstrassPoint<S> | BLSInput)[]): WeierstrassPoint<S>;
    hash(message: Uint8Array, DST?: string | Uint8Array, hashOpts?: H2CHashOpts): WeierstrassPoint<S>;
    Signature: BlsLongSignatureCoder<S>;
}
type BlsSignatureCoders = Partial<{
    LongSignature: BlsLongSignatureCoder<Fp2>;
    ShortSignature: BlsShortSignatureCoder<Fp>;
}>;
export declare function blsBasic(fields: BlsFields, G1_Point: WeierstrassPointCons<Fp>, G2_Point: WeierstrassPointCons<Fp2>, params: BlsPairingParams): BlsCurvePair;
export declare function bls(fields: BlsFields, G1_Point: WeierstrassPointCons<Fp>, G2_Point: WeierstrassPointCons<Fp2>, params: BlsPairingParams, hasherParams: BlsHasherParams, signatureCoders: BlsSignatureCoders): BlsCurvePairWithSignatures;
export {};
//# sourceMappingURL=bls.d.ts.map