"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u8aToFloat = u8aToFloat;
/**
 * @name u8aToFloat
 * @description Converts a Uint8Array value into the float (either 32 or 64-bit)
 * representation.
 */
function u8aToFloat(value, { bitLength = 32, isLe = true } = {}) {
    if (bitLength !== 32 && bitLength !== 64) {
        throw new Error('Invalid bitLength provided, expected 32 or 64');
    }
    else if (value.length < (bitLength / 8)) {
        throw new Error(`Invalid input buffer provided, expected at least ${bitLength / 8} bytes, found ${value.length}`);
    }
    const dv = new DataView(value.buffer, value.byteOffset);
    return bitLength === 32
        ? dv.getFloat32(0, isLe)
        : dv.getFloat64(0, isLe);
}
