import type { ToBnOptions } from '../types.js';
/**
 * @name hexToBigInt
 * @summary Creates a BigInt instance object from a hex string.
 */
export declare function hexToBigInt(value?: string | null, { isLe, isNegative }?: ToBnOptions): bigint;
