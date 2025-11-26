/**
 * NIST secp384r1 aka p384.
 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import { type HTFMethod } from './abstract/hash-to-curve.ts';
import { p384 as p384n } from './nist.ts';
export declare const p384: typeof p384n;
export declare const secp384r1: typeof p384n;
export declare const hashToCurve: HTFMethod<bigint>;
export declare const encodeToCurve: HTFMethod<bigint>;
/** @deprecated Use `import { p384_hasher } from "@noble/curves/nist"` module. */
//# sourceMappingURL=p384.d.ts.map