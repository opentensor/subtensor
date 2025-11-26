import type { BN } from '../bn/index.js';
import type { ToBigInt, ToBn } from '../types.js';
/**
 * @name nSqrt
 * @summary Calculates the integer square root of a bigint
 */
export declare function nSqrt<ExtToBn extends ToBn | ToBigInt>(value: ExtToBn | BN | bigint | string | number | null): bigint;
