"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hexToU8a = hexToU8a;
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
function hexToU8a(_value, bitLength = -1) {
    if (!_value) {
        return new Uint8Array();
    }
    const value = _value.startsWith('0x')
        ? _value.substring(2)
        : _value;
    const buf = Buffer.from(value, 'hex');
    const valLength = value.length / 2;
    const resultLength = Math.ceil(bitLength === -1
        ? valLength
        : bitLength / 8);
    if (resultLength === valLength) {
        return Uint8Array.from(buf);
    }
    const offset = resultLength > valLength
        ? resultLength - valLength
        : 0;
    if (offset) {
        const u8a = new Uint8Array(resultLength);
        u8a.set(buf, offset);
        return u8a;
    }
    return Uint8Array.from(buf.subarray(0, resultLength));
}
