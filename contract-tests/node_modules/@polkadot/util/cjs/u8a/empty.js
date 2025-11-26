"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u8aEmpty = u8aEmpty;
/**
 * @name u8aEmpty
 * @summary Tests for a `Uint8Array` for emptyness
 * @description
 * Checks to see if the input `Uint8Array` has zero length or contains all 0 values.
 */
function u8aEmpty(value) {
    const len = value.length | 0;
    // on smaller sizes, the byte-by-byte compare is faster than allocating
    // another object for DataView (on very large arrays the DataView is faster)
    for (let i = 0; i < len; i++) {
        if (value[i] | 0) {
            return false;
        }
    }
    return true;
}
