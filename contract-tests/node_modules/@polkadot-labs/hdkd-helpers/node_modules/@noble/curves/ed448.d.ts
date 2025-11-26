import type { AffinePoint } from './abstract/curve.ts';
import { PrimeEdwardsPoint, type EdDSA, type EdwardsPoint, type EdwardsPointCons } from './abstract/edwards.ts';
import { type H2CHasher, type H2CHasherBase } from './abstract/hash-to-curve.ts';
import { type IField } from './abstract/modular.ts';
import { type MontgomeryECDH } from './abstract/montgomery.ts';
import { type OPRF } from './abstract/oprf.ts';
/**
 * ed448 EdDSA curve and methods.
 * @example
 * ```js
 * import { ed448 } from '@noble/curves/ed448.js';
 * const { secretKey, publicKey } = ed448.keygen();
 * // const publicKey = ed448.getPublicKey(secretKey);
 * const msg = new TextEncoder().encode('hello noble');
 * const sig = ed448.sign(msg, secretKey);
 * const isValid = ed448.verify(sig, msg, publicKey);
 * ```
 */
export declare const ed448: EdDSA;
/** Prehashed version of ed448. See {@link ed448} */
export declare const ed448ph: EdDSA;
/**
 * E448 (NIST) != edwards448 used in ed448.
 * E448 is birationally equivalent to edwards448.
 */
export declare const E448: EdwardsPointCons;
/**
 * ECDH using curve448 aka x448.
 *
 * @example
 * ```js
 * import { x448 } from '@noble/curves/ed448.js';
 * const alice = x448.keygen();
 * const bob = x448.keygen();
 * const shared = x448.getSharedSecret(alice.secretKey, bob.publicKey);
 * ```
 */
export declare const x448: MontgomeryECDH;
/** Hashing / encoding to ed448 points / field. RFC 9380 methods. */
export declare const ed448_hasher: H2CHasher<EdwardsPointCons>;
/**
 * Each ed448/EdwardsPoint has 4 different equivalent points. This can be
 * a source of bugs for protocols like ring signatures. Decaf was created to solve this.
 * Decaf point operates in X:Y:Z:T extended coordinates like EdwardsPoint,
 * but it should work in its own namespace: do not combine those two.
 * See [RFC9496](https://www.rfc-editor.org/rfc/rfc9496).
 */
declare class _DecafPoint extends PrimeEdwardsPoint<_DecafPoint> {
    static BASE: _DecafPoint;
    static ZERO: _DecafPoint;
    static Fp: IField<bigint>;
    static Fn: IField<bigint>;
    constructor(ep: EdwardsPoint);
    static fromAffine(ap: AffinePoint<bigint>): _DecafPoint;
    protected assertSame(other: _DecafPoint): void;
    protected init(ep: EdwardsPoint): _DecafPoint;
    static fromBytes(bytes: Uint8Array): _DecafPoint;
    /**
     * Converts decaf-encoded string to decaf point.
     * Described in [RFC9496](https://www.rfc-editor.org/rfc/rfc9496#name-decode-2).
     * @param hex Decaf-encoded 56 bytes. Not every 56-byte string is valid decaf encoding
     */
    static fromHex(hex: string): _DecafPoint;
    /**
     * Encodes decaf point to Uint8Array.
     * Described in [RFC9496](https://www.rfc-editor.org/rfc/rfc9496#name-encode-2).
     */
    toBytes(): Uint8Array;
    /**
     * Compare one point to another.
     * Described in [RFC9496](https://www.rfc-editor.org/rfc/rfc9496#name-equals-2).
     */
    equals(other: _DecafPoint): boolean;
    is0(): boolean;
}
export declare const decaf448: {
    Point: typeof _DecafPoint;
};
/** Hashing to decaf448 points / field. RFC 9380 methods. */
export declare const decaf448_hasher: H2CHasherBase<typeof _DecafPoint>;
/** decaf448 OPRF, defined in RFC 9497. */
export declare const decaf448_oprf: OPRF;
/**
 * Weird / bogus points, useful for debugging.
 * Unlike ed25519, there is no ed448 generator point which can produce full T subgroup.
 * Instead, there is a Klein four-group, which spans over 2 independent 2-torsion points:
 * (0, 1), (0, -1), (-1, 0), (1, 0).
 */
export declare const ED448_TORSION_SUBGROUP: string[];
export {};
//# sourceMappingURL=ed448.d.ts.map