import { hexToU8a } from '../hex/toU8a.js';
import { numberToHex } from './toHex.js';
/**
 * @name numberToU8a
 * @summary Creates a Uint8Array object from a number.
 * @description
 * `null`/`undefined`/`NaN` inputs returns an empty `Uint8Array` result. `number` input values return the actual bytes value converted to a `Uint8Array`. With `bitLength`, it converts the value to the equivalent size.
 * @example
 * <BR>
 *
 * ```javascript
 * import { numberToU8a } from '@polkadot/util';
 *
 * numberToU8a(0x1234); // => [0x12, 0x34]
 * ```
 */
export function numberToU8a(value, bitLength = -1) {
    return hexToU8a(numberToHex(value, bitLength));
}
