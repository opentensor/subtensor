import { type EdDSA, type EdwardsPoint } from './abstract/edwards.ts';
import { type ECDSA } from './abstract/weierstrass.ts';
/** Curve over scalar field of bls12-381. jubjub Fp = bls n */
export declare const jubjub: EdDSA;
/** Curve over scalar field of bn254. babyjubjub Fp = bn254 n */
export declare const babyjubjub: EdDSA;
export declare function jubjub_groupHash(tag: Uint8Array, personalization: Uint8Array): EdwardsPoint;
export declare function jubjub_findGroupHash(m: Uint8Array, personalization: Uint8Array): EdwardsPoint;
/** Brainpool P256r1 with sha256, from RFC 5639. */
export declare const brainpoolP256r1: ECDSA;
/** Brainpool P384r1 with sha384, from RFC 5639. */
export declare const brainpoolP384r1: ECDSA;
/** Brainpool P521r1 with sha512, from RFC 5639. */
export declare const brainpoolP512r1: ECDSA;
//# sourceMappingURL=misc.d.ts.map