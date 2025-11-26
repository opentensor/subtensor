const CHR = '0123456789abcdef';
const U8 = new Uint8Array(256);
const U16 = new Uint8Array(256 * 256);
for (let i = 0, count = CHR.length; i < count; i++) {
    U8[CHR[i].charCodeAt(0) | 0] = i | 0;
    if (i > 9) {
        U8[CHR[i].toUpperCase().charCodeAt(0) | 0] = i | 0;
    }
}
for (let i = 0; i < 256; i++) {
    const s = i << 8;
    for (let j = 0; j < 256; j++) {
        U16[s | j] = (U8[i] << 4) | U8[j];
    }
}
/**
 * @name hexToU8a
 * @summary Creates a Uint8Array object from a hex string.
 * @description
 * `null` inputs returns an empty `Uint8Array` result. Hex input values return the actual bytes value converted to a Uint8Array. Anything that is not a hex string (including the `0x` prefix) throws an error.
 * @example
 * <BR>
 *
 * ```javascript
 * import { hexToU8a } from '@polkadot/util';
 *
 * hexToU8a('0x80001f'); // Uint8Array([0x80, 0x00, 0x1f])
 * hexToU8a('0x80001f', 32); // Uint8Array([0x00, 0x80, 0x00, 0x1f])
 * ```
 */
export function hexToU8a(value, bitLength = -1) {
    if (!value) {
        return new Uint8Array();
    }
    let s = value.startsWith('0x')
        ? 2
        : 0;
    const decLength = Math.ceil((value.length - s) / 2);
    const endLength = Math.ceil(bitLength === -1
        ? decLength
        : bitLength / 8);
    const result = new Uint8Array(endLength);
    const offset = endLength > decLength
        ? endLength - decLength
        : 0;
    for (let i = offset; i < endLength; i++, s += 2) {
        // The big factor here is actually the string lookups. If we do
        // HEX_TO_U16[value.substring()] we get an 10x slowdown. In the
        // same vein using charCodeAt (as opposed to value[s] or value.charAt(s)) is
        // also the faster operation by at least 2x with the character map above
        result[i] = U16[(value.charCodeAt(s) << 8) | value.charCodeAt(s + 1)];
    }
    return result;
}
