import { u8aToString } from '../u8a/toString.js';
import { hexToU8a } from './toU8a.js';
/**
 * @name hexToU8a
 * @summary Creates a Uint8Array object from a hex string.
 * @description
 * Hex input values return the actual bytes value converted to a string. Anything that is not a hex string (including the `0x` prefix) throws an error.
 * @example
 * <BR>
 *
 * ```javascript
 * import { hexToString } from '@polkadot/util';
 *
 * hexToU8a('0x68656c6c6f'); // hello
 * ```
 */
export function hexToString(_value) {
    return u8aToString(hexToU8a(_value));
}
