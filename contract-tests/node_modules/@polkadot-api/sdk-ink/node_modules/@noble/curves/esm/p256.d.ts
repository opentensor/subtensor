/**
 * NIST secp256r1 aka p256.
 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import { type HTFMethod } from './abstract/hash-to-curve.ts';
import { p256 as p256n } from './nist.ts';
export declare const p256: typeof p256n;
export declare const secp256r1: typeof p256n;
export declare const hashToCurve: HTFMethod<bigint>;
export declare const encodeToCurve: HTFMethod<bigint>;
//# sourceMappingURL=p256.d.ts.map