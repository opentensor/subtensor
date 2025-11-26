"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u8aToHex = u8aToHex;
const U8 = new Array(256);
const U16 = new Array(256 * 256);
for (let n = 0; n < 256; n++) {
    U8[n] = n.toString(16).padStart(2, '0');
}
for (let i = 0; i < 256; i++) {
    const s = i << 8;
    for (let j = 0; j < 256; j++) {
        U16[s | j] = U8[i] + U8[j];
    }
}
/** @internal */
function hex(value, result) {
    const mod = (value.length % 2) | 0;
    const length = (value.length - mod) | 0;
    for (let i = 0; i < length; i += 2) {
        result += U16[(value[i] << 8) | value[i + 1]];
    }
    if (mod) {
        result += U8[value[length] | 0];
    }
    return result;
}
/**
 * @name u8aToHex
 * @summary Creates a hex string from a Uint8Array object.
 * @description
 * `UInt8Array` input values return the actual hex string. `null` or `undefined` values returns an `0x` string.
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aToHex } from '@polkadot/util';
 *
 * u8aToHex(new Uint8Array([0x68, 0x65, 0x6c, 0x6c, 0xf])); // 0x68656c0f
 * ```
 */
function u8aToHex(value, bitLength = -1, isPrefixed = true) {
    // this is not 100% correct sinmce we support isPrefixed = false....
    const empty = isPrefixed
        ? '0x'
        : '';
    if (!value?.length) {
        return empty;
    }
    else if (bitLength > 0) {
        const length = Math.ceil(bitLength / 8);
        if (value.length > length) {
            return `${hex(value.subarray(0, length / 2), empty)}…${hex(value.subarray(value.length - length / 2), '')}`;
        }
    }
    return hex(value, empty);
}
