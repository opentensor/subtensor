import { type AffinePoint } from './abstract/curve.ts';
import { PrimeEdwardsPoint, type EdDSA, type EdwardsPoint, type EdwardsPointCons } from './abstract/edwards.ts';
import { type H2CHasher, type H2CHasherBase } from './abstract/hash-to-curve.ts';
import { type IField } from './abstract/modular.ts';
import { type MontgomeryECDH } from './abstract/montgomery.ts';
import { type OPRF } from './abstract/oprf.ts';
/**
 * ed25519 curve with EdDSA signatures.
 * @example
 * ```js
 * import { ed25519 } from '@noble/curves/ed25519.js';
 * const { secretKey, publicKey } = ed25519.keygen();
 * // const publicKey = ed25519.getPublicKey(secretKey);
 * const msg = new TextEncoder().encode('hello noble');
 * const sig = ed25519.sign(msg, secretKey);
 * const isValid = ed25519.verify(sig, msg, pub); // ZIP215
 * // RFC8032 / FIPS 186-5
 * const isValid2 = ed25519.verify(sig, msg, pub, { zip215: false });
 * ```
 */
export declare const ed25519: EdDSA;
/** Context version of ed25519 (ctx for domain separation). See {@link ed25519} */
export declare const ed25519ctx: EdDSA;
/** Prehashed version of ed25519. See {@link ed25519} */
export declare const ed25519ph: EdDSA;
/**
 * ECDH using curve25519 aka x25519.
 * @example
 * ```js
 * import { x25519 } from '@noble/curves/ed25519.js';
 * const alice = x25519.keygen();
 * const bob = x25519.keygen();
 * const shared = x25519.getSharedSecret(alice.secretKey, bob.publicKey);
 * ```
 */
export declare const x25519: MontgomeryECDH;
/**
 * RFC 9380 method `map_to_curve_elligator2_curve25519`. Experimental name: may be renamed later.
 * @private
 */
export declare function _map_to_curve_elligator2_curve25519(u: bigint): {
    xMn: bigint;
    xMd: bigint;
    yMn: bigint;
    yMd: bigint;
};
/** Hashing to ed25519 points / field. RFC 9380 methods. */
export declare const ed25519_hasher: H2CHasher<EdwardsPointCons>;
/**
 * Wrapper over Edwards Point for ristretto255.
 *
 * Each ed25519/EdwardsPoint has 8 different equivalent points. This can be
 * a source of bugs for protocols like ring signatures. Ristretto was created to solve this.
 * Ristretto point operates in X:Y:Z:T extended coordinates like EdwardsPoint,
 * but it should work in its own namespace: do not combine those two.
 * See [RFC9496](https://www.rfc-editor.org/rfc/rfc9496).
 */
declare class _RistrettoPoint extends PrimeEdwardsPoint<_RistrettoPoint> {
    static BASE: _RistrettoPoint;
    static ZERO: _RistrettoPoint;
    static Fp: IField<bigint>;
    static Fn: IField<bigint>;
    constructor(ep: EdwardsPoint);
    static fromAffine(ap: AffinePoint<bigint>): _RistrettoPoint;
    protected assertSame(other: _RistrettoPoint): void;
    protected init(ep: EdwardsPoint): _RistrettoPoint;
    static fromBytes(bytes: Uint8Array): _RistrettoPoint;
    /**
     * Converts ristretto-encoded string to ristretto point.
     * Described in [RFC9496](https://www.rfc-editor.org/rfc/rfc9496#name-decode).
     * @param hex Ristretto-encoded 32 bytes. Not every 32-byte string is valid ristretto encoding
     */
    static fromHex(hex: string): _RistrettoPoint;
    /**
     * Encodes ristretto point to Uint8Array.
     * Described in [RFC9496](https://www.rfc-editor.org/rfc/rfc9496#name-encode).
     */
    toBytes(): Uint8Array;
    /**
     * Compares two Ristretto points.
     * Described in [RFC9496](https://www.rfc-editor.org/rfc/rfc9496#name-equals).
     */
    equals(other: _RistrettoPoint): boolean;
    is0(): boolean;
}
export declare const ristretto255: {
    Point: typeof _RistrettoPoint;
};
/** Hashing to ristretto255 points / field. RFC 9380 methods. */
export declare const ristretto255_hasher: H2CHasherBase<typeof _RistrettoPoint>;
/** ristretto255 OPRF, defined in RFC 9497. */
export declare const ristretto255_oprf: OPRF;
/**
 * Weird / bogus points, useful for debugging.
 * All 8 ed25519 points of 8-torsion subgroup can be generated from the point
 * T = `26e8958fc2b227b045c3f489f2ef98f0d5dfac05d3c63339b13802886d53fc05`.
 * ⟨T⟩ = { O, T, 2T, 3T, 4T, 5T, 6T, 7T }
 */
export declare const ED25519_TORSION_SUBGROUP: string[];
export {};
//# sourceMappingURL=ed25519.d.ts.map