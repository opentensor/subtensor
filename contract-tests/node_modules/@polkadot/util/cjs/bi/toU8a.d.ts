import type { BN } from '../bn/bn.js';
import type { NumberOptions, ToBigInt, ToBn } from '../types.js';
/**
 * @name nToU8a
 * @summary Creates a Uint8Array object from a bigint.
 */
export declare function nToU8a<ExtToBn extends ToBn | ToBigInt>(value?: ExtToBn | BN | bigint | number | null, { bitLength, isLe, isNegative }?: NumberOptions): Uint8Array;
