import { type CurveFnWithCreate } from './_shortw_utils.ts';
import { type Hasher } from './abstract/hash-to-curve.ts';
/**
 * secp256r1 curve, ECDSA and ECDH methods.
 * Field: `2n**224n * (2n**32n-1n) + 2n**192n + 2n**96n-1n`
 */
export declare const p256: CurveFnWithCreate;
/** Alias to p256. */
export declare const secp256r1: CurveFnWithCreate;
/** Hashing / encoding to p256 points / field. RFC 9380 methods. */
export declare const p256_hasher: Hasher<bigint>;
/**
 * secp384r1 curve, ECDSA and ECDH methods.
 * Field: `2n**384n - 2n**128n - 2n**96n + 2n**32n - 1n`.
 * */
export declare const p384: CurveFnWithCreate;
/** Alias to p384. */
export declare const secp384r1: CurveFnWithCreate;
/** Hashing / encoding to p384 points / field. RFC 9380 methods. */
export declare const p384_hasher: Hasher<bigint>;
/**
 * NIST secp521r1 aka p521 curve, ECDSA and ECDH methods.
 * Field: `2n**521n - 1n`.
 */
export declare const p521: CurveFnWithCreate;
/** Alias to p521. */
export declare const secp521r1: CurveFnWithCreate;
/** Hashing / encoding to p521 points / field. RFC 9380 methods. */
export declare const p521_hasher: Hasher<bigint>;
//# sourceMappingURL=nist.d.ts.map