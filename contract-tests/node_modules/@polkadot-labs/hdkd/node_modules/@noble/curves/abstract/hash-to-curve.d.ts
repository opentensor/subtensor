/**
 * hash-to-curve from RFC 9380.
 * Hashes arbitrary-length byte strings to a list of one or more elements of a finite field F.
 * https://www.rfc-editor.org/rfc/rfc9380
 * @module
 */
/*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) */
import type { CHash } from '../utils.ts';
import type { AffinePoint, PC_ANY, PC_F, PC_P } from './curve.ts';
import { type IField } from './modular.ts';
export type AsciiOrBytes = string | Uint8Array;
/**
 * * `DST` is a domain separation tag, defined in section 2.2.5
 * * `p` characteristic of F, where F is a finite field of characteristic p and order q = p^m
 * * `m` is extension degree (1 for prime fields)
 * * `k` is the target security target in bits (e.g. 128), from section 5.1
 * * `expand` is `xmd` (SHA2, SHA3, BLAKE) or `xof` (SHAKE, BLAKE-XOF)
 * * `hash` conforming to `utils.CHash` interface, with `outputLen` / `blockLen` props
 */
export type H2COpts = {
    DST: AsciiOrBytes;
    expand: 'xmd' | 'xof';
    hash: CHash;
    p: bigint;
    m: number;
    k: number;
};
export type H2CHashOpts = {
    expand: 'xmd' | 'xof';
    hash: CHash;
};
export type MapToCurve<T> = (scalar: bigint[]) => AffinePoint<T>;
export type H2CDSTOpts = {
    DST: AsciiOrBytes;
};
export type H2CHasherBase<PC extends PC_ANY> = {
    hashToCurve(msg: Uint8Array, options?: H2CDSTOpts): PC_P<PC>;
    hashToScalar(msg: Uint8Array, options?: H2CDSTOpts): bigint;
    deriveToCurve?(msg: Uint8Array, options?: H2CDSTOpts): PC_P<PC>;
    Point: PC;
};
/**
 * RFC 9380 methods, with cofactor clearing. See https://www.rfc-editor.org/rfc/rfc9380#section-3.
 *
 * * hashToCurve: `map(hash(input))`, encodes RANDOM bytes to curve (WITH hashing)
 * * encodeToCurve: `map(hash(input))`, encodes NON-UNIFORM bytes to curve (WITH hashing)
 * * mapToCurve: `map(scalars)`, encodes NON-UNIFORM scalars to curve (NO hashing)
 */
export type H2CHasher<PC extends PC_ANY> = H2CHasherBase<PC> & {
    encodeToCurve(msg: Uint8Array, options?: H2CDSTOpts): PC_P<PC>;
    mapToCurve: MapToCurve<PC_F<PC>>;
    defaults: H2COpts & {
        encodeDST?: AsciiOrBytes;
    };
};
/**
 * Produces a uniformly random byte string using a cryptographic hash function H that outputs b bits.
 * [RFC 9380 5.3.1](https://www.rfc-editor.org/rfc/rfc9380#section-5.3.1).
 */
export declare function expand_message_xmd(msg: Uint8Array, DST: AsciiOrBytes, lenInBytes: number, H: CHash): Uint8Array;
/**
 * Produces a uniformly random byte string using an extendable-output function (XOF) H.
 * 1. The collision resistance of H MUST be at least k bits.
 * 2. H MUST be an XOF that has been proved indifferentiable from
 *    a random oracle under a reasonable cryptographic assumption.
 * [RFC 9380 5.3.2](https://www.rfc-editor.org/rfc/rfc9380#section-5.3.2).
 */
export declare function expand_message_xof(msg: Uint8Array, DST: AsciiOrBytes, lenInBytes: number, k: number, H: CHash): Uint8Array;
/**
 * Hashes arbitrary-length byte strings to a list of one or more elements of a finite field F.
 * [RFC 9380 5.2](https://www.rfc-editor.org/rfc/rfc9380#section-5.2).
 * @param msg a byte string containing the message to hash
 * @param count the number of elements of F to output
 * @param options `{DST: string, p: bigint, m: number, k: number, expand: 'xmd' | 'xof', hash: H}`, see above
 * @returns [u_0, ..., u_(count - 1)], a list of field elements.
 */
export declare function hash_to_field(msg: Uint8Array, count: number, options: H2COpts): bigint[][];
type XY<T> = (x: T, y: T) => {
    x: T;
    y: T;
};
type XYRatio<T> = [T[], T[], T[], T[]];
export declare function isogenyMap<T, F extends IField<T>>(field: F, map: XYRatio<T>): XY<T>;
export declare const _DST_scalar: Uint8Array;
/** Creates hash-to-curve methods from EC Point and mapToCurve function. See {@link H2CHasher}. */
export declare function createHasher<PC extends PC_ANY>(Point: PC, mapToCurve: MapToCurve<PC_F<PC>>, defaults: H2COpts & {
    encodeDST?: AsciiOrBytes;
}): H2CHasher<PC>;
export {};
//# sourceMappingURL=hash-to-curve.d.ts.map