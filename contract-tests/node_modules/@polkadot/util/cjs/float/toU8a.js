"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.floatToU8a = floatToU8a;
/**
 * @name floatToU8a
 * @description Converts a float into a U8a representation (While we don't use BE in SCALE
 * we still allow for either representation, although, as elsewhere, isLe is default)
 */
function floatToU8a(value = 0.0, { bitLength = 32, isLe = true } = {}) {
    if (bitLength !== 32 && bitLength !== 64) {
        throw new Error('Invalid bitLength provided, expected 32 or 64');
    }
    const result = new Uint8Array(bitLength / 8);
    const dv = new DataView(result.buffer, result.byteOffset);
    if (bitLength === 32) {
        dv.setFloat32(0, Number(value), isLe);
    }
    else {
        dv.setFloat64(0, Number(value), isLe);
    }
    return result;
}
