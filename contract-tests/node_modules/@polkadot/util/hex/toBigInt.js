import { BigInt } from '@polkadot/x-bigint';
import { u8aToBigInt } from '../u8a/toBigInt.js';
import { hexToU8a } from './toU8a.js';
/**
 * @name hexToBigInt
 * @summary Creates a BigInt instance object from a hex string.
 */
export function hexToBigInt(value, { isLe = false, isNegative = false } = {}) {
    return !value || value === '0x'
        ? BigInt(0)
        : u8aToBigInt(hexToU8a(value), { isLe, isNegative });
}
