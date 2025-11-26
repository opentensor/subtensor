import type { ToBnOptions } from '../types.js';
/**
 * @name u8aToBigInt
 * @summary Creates a BigInt from a Uint8Array object.
 */
export declare function u8aToBigInt(value: Uint8Array, { isLe, isNegative }?: ToBnOptions): bigint;
