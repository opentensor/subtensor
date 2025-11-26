export { abytes, anumber, bytesToHex, concatBytes, hexToBytes, isBytes, randomBytes, } from '@noble/hashes/utils.js';
export type CHash = {
    (message: Uint8Array): Uint8Array;
    blockLen: number;
    outputLen: number;
    create(opts?: {
        dkLen?: number;
    }): any;
};
export type FHash = (message: Uint8Array) => Uint8Array;
export declare function abool(value: boolean, title?: string): boolean;
export declare function asafenumber(value: number, title?: string): void;
export declare function numberToHexUnpadded(num: number | bigint): string;
export declare function hexToNumber(hex: string): bigint;
export declare function bytesToNumberBE(bytes: Uint8Array): bigint;
export declare function bytesToNumberLE(bytes: Uint8Array): bigint;
export declare function numberToBytesBE(n: number | bigint, len: number): Uint8Array;
export declare function numberToBytesLE(n: number | bigint, len: number): Uint8Array;
export declare function numberToVarBytesBE(n: number | bigint): Uint8Array;
export declare function equalBytes(a: Uint8Array, b: Uint8Array): boolean;
/**
 * Copies Uint8Array. We can't use u8a.slice(), because u8a can be Buffer,
 * and Buffer#slice creates mutable copy. Never use Buffers!
 */
export declare function copyBytes(bytes: Uint8Array): Uint8Array;
/**
 * Decodes 7-bit ASCII string to Uint8Array, throws on non-ascii symbols
 * Should be safe to use for things expected to be ASCII.
 * Returns exact same result as `TextEncoder` for ASCII or throws.
 */
export declare function asciiToBytes(ascii: string): Uint8Array;
export declare function inRange(n: bigint, min: bigint, max: bigint): boolean;
/**
 * Asserts min <= n < max. NOTE: It's < max and not <= max.
 * @example
 * aInRange('x', x, 1n, 256n); // would assume x is in (1n..255n)
 */
export declare function aInRange(title: string, n: bigint, min: bigint, max: bigint): void;
/**
 * Calculates amount of bits in a bigint.
 * Same as `n.toString(2).length`
 * TODO: merge with nLength in modular
 */
export declare function bitLen(n: bigint): number;
/**
 * Gets single bit at position.
 * NOTE: first bit position is 0 (same as arrays)
 * Same as `!!+Array.from(n.toString(2)).reverse()[pos]`
 */
export declare function bitGet(n: bigint, pos: number): bigint;
/**
 * Sets single bit at position.
 */
export declare function bitSet(n: bigint, pos: number, value: boolean): bigint;
/**
 * Calculate mask for N bits. Not using ** operator with bigints because of old engines.
 * Same as BigInt(`0b${Array(i).fill('1').join('')}`)
 */
export declare const bitMask: (n: number) => bigint;
type Pred<T> = (v: Uint8Array) => T | undefined;
/**
 * Minimal HMAC-DRBG from NIST 800-90 for RFC6979 sigs.
 * @returns function that will call DRBG until 2nd arg returns something meaningful
 * @example
 *   const drbg = createHmacDRBG<Key>(32, 32, hmac);
 *   drbg(seed, bytesToKey); // bytesToKey must return Key or undefined
 */
export declare function createHmacDrbg<T>(hashLen: number, qByteLen: number, hmacFn: (key: Uint8Array, message: Uint8Array) => Uint8Array): (seed: Uint8Array, predicate: Pred<T>) => T;
export declare function validateObject(object: Record<string, any>, fields?: Record<string, string>, optFields?: Record<string, string>): void;
/**
 * throws not implemented error
 */
export declare const notImplemented: () => never;
/**
 * Memoizes (caches) computation result.
 * Uses WeakMap: the value is going auto-cleaned by GC after last reference is removed.
 */
export declare function memoized<T extends object, R, O extends any[]>(fn: (arg: T, ...args: O) => R): (arg: T, ...args: O) => R;
export interface CryptoKeys {
    lengths: {
        seed?: number;
        public?: number;
        secret?: number;
    };
    keygen: (seed?: Uint8Array) => {
        secretKey: Uint8Array;
        publicKey: Uint8Array;
    };
    getPublicKey: (secretKey: Uint8Array) => Uint8Array;
}
/** Generic interface for signatures. Has keygen, sign and verify. */
export interface Signer extends CryptoKeys {
    lengths: {
        seed?: number;
        public?: number;
        secret?: number;
        signRand?: number;
        signature?: number;
    };
    sign: (msg: Uint8Array, secretKey: Uint8Array) => Uint8Array;
    verify: (sig: Uint8Array, msg: Uint8Array, publicKey: Uint8Array) => boolean;
}
//# sourceMappingURL=utils.d.ts.map