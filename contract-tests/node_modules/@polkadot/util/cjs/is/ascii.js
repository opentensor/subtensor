"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isAscii = isAscii;
const toU8a_js_1 = require("../u8a/toU8a.js");
const hex_js_1 = require("./hex.js");
const string_js_1 = require("./string.js");
/** @internal */
function isAsciiStr(str) {
    for (let i = 0, count = str.length; i < count; i++) {
        const b = str.charCodeAt(i);
        // check is inlined here, it is faster than making a call
        if (b < 32 || b > 126) {
            return false;
        }
    }
    return true;
}
/** @internal */
function isAsciiBytes(u8a) {
    for (let i = 0, count = u8a.length; i < count; i++) {
        const b = u8a[i] | 0;
        // check is inlined here, it is faster than making a call
        if (b < 32 || b > 126) {
            return false;
        }
    }
    return true;
}
/**
 * @name isAscii
 * @summary Tests if the input is printable ASCII
 * @description
 * Checks to see if the input string or Uint8Array is printable ASCII, 32-127 + formatters
 */
function isAscii(value) {
    return (0, string_js_1.isString)(value)
        ? (0, hex_js_1.isHex)(value)
            ? isAsciiBytes((0, toU8a_js_1.u8aToU8a)(value))
            : isAsciiStr(value)
        : value
            ? isAsciiBytes(value)
            : false;
}
