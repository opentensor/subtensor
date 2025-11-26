import { type H2CHasher } from './abstract/hash-to-curve.ts';
import { type OPRF } from './abstract/oprf.ts';
import { type ECDSA, type WeierstrassPointCons } from './abstract/weierstrass.ts';
/**
 * NIST P256 (aka secp256r1, prime256v1) curve, ECDSA and ECDH methods.
 * Hashes inputs with sha256 by default.
 *
 * @example
 * ```js
 * import { p256 } from '@noble/curves/nist.js';
 * const { secretKey, publicKey } = p256.keygen();
 * // const publicKey = p256.getPublicKey(secretKey);
 * const msg = new TextEncoder().encode('hello noble');
 * const sig = p256.sign(msg, secretKey);
 * const isValid = p256.verify(sig, msg, publicKey);
 * // const sigKeccak = p256.sign(keccak256(msg), secretKey, { prehash: false });
 * ```
 */
export declare const p256: ECDSA;
/** Hashing / encoding to p256 points / field. RFC 9380 methods. */
export declare const p256_hasher: H2CHasher<WeierstrassPointCons<bigint>>;
/** p256 OPRF, defined in RFC 9497. */
export declare const p256_oprf: OPRF;
/** NIST P384 (aka secp384r1) curve, ECDSA and ECDH methods. Hashes inputs with sha384 by default. */
export declare const p384: ECDSA;
/** Hashing / encoding to p384 points / field. RFC 9380 methods. */
export declare const p384_hasher: H2CHasher<WeierstrassPointCons<bigint>>;
/** p384 OPRF, defined in RFC 9497. */
export declare const p384_oprf: OPRF;
/** NIST P521 (aka secp521r1) curve, ECDSA and ECDH methods. Hashes inputs with sha512 by default. */
export declare const p521: ECDSA;
/** Hashing / encoding to p521 points / field. RFC 9380 methods. */
export declare const p521_hasher: H2CHasher<WeierstrassPointCons<bigint>>;
/** p521 OPRF, defined in RFC 9497. */
export declare const p521_oprf: OPRF;
//# sourceMappingURL=nist.d.ts.map