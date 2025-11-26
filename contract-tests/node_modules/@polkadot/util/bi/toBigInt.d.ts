import type { BN } from '../bn/bn.js';
import type { ToBigInt, ToBn } from '../types.js';
/**
 * @name nToBigInt
 * @summary Creates a bigInt value from a BN, bigint, string (base 10 or hex) or number input.
 */
export declare function nToBigInt<ExtToBn extends ToBigInt | ToBn>(value?: ExtToBn | BN | bigint | string | number | null): bigint;
