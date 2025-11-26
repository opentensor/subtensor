import { u8aToHex } from '../u8a/index.js';
import { nToU8a } from './toU8a.js';
/**
 * @name nToHex
 * @summary Creates a hex value from a bigint object.
 */
export function nToHex(value, { bitLength = -1, isLe = false, isNegative = false } = {}) {
    return u8aToHex(nToU8a(value || 0, { bitLength, isLe, isNegative }));
}
