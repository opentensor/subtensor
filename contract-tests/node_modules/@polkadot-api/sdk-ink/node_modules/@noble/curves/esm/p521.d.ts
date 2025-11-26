/**
 * NIST secp521r1 aka p521.
 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import { type HTFMethod } from './abstract/hash-to-curve.ts';
import { p521 as p521n } from './nist.ts';
export declare const p521: typeof p521n;
export declare const secp521r1: typeof p521n;
export declare const hashToCurve: HTFMethod<bigint>;
export declare const encodeToCurve: HTFMethod<bigint>;
//# sourceMappingURL=p521.d.ts.map