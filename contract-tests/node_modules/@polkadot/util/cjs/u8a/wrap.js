"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.U8A_WRAP_POSTFIX = exports.U8A_WRAP_PREFIX = exports.U8A_WRAP_ETHEREUM = void 0;
exports.u8aIsWrapped = u8aIsWrapped;
exports.u8aUnwrapBytes = u8aUnwrapBytes;
exports.u8aWrapBytes = u8aWrapBytes;
const concat_js_1 = require("./concat.js");
const eq_js_1 = require("./eq.js");
const toU8a_js_1 = require("./toU8a.js");
/** @internal */
exports.U8A_WRAP_ETHEREUM = (0, toU8a_js_1.u8aToU8a)('\x19Ethereum Signed Message:\n');
/** @internal */
exports.U8A_WRAP_PREFIX = (0, toU8a_js_1.u8aToU8a)('<Bytes>');
/** @internal */
exports.U8A_WRAP_POSTFIX = (0, toU8a_js_1.u8aToU8a)('</Bytes>');
const WRAP_LEN = exports.U8A_WRAP_PREFIX.length + exports.U8A_WRAP_POSTFIX.length;
/** @internal */
function u8aIsWrapped(u8a, withEthereum) {
    return ((u8a.length >= WRAP_LEN &&
        (0, eq_js_1.u8aEq)(u8a.subarray(0, exports.U8A_WRAP_PREFIX.length), exports.U8A_WRAP_PREFIX) &&
        (0, eq_js_1.u8aEq)(u8a.slice(-exports.U8A_WRAP_POSTFIX.length), exports.U8A_WRAP_POSTFIX)) ||
        (withEthereum &&
            u8a.length >= exports.U8A_WRAP_ETHEREUM.length &&
            (0, eq_js_1.u8aEq)(u8a.subarray(0, exports.U8A_WRAP_ETHEREUM.length), exports.U8A_WRAP_ETHEREUM)));
}
/**
 * @name u8aUnwrapBytes
 * @description Removes all <Bytes>...</Bytes> wrappers from the supplied value
 */
function u8aUnwrapBytes(bytes) {
    const u8a = (0, toU8a_js_1.u8aToU8a)(bytes);
    // we don't want to unwrap Ethereum-style wraps
    return u8aIsWrapped(u8a, false)
        ? u8a.subarray(exports.U8A_WRAP_PREFIX.length, u8a.length - exports.U8A_WRAP_POSTFIX.length)
        : u8a;
}
/**
 * @name u8aWrapBytes
 * @description
 * Adds a <Bytes>...</Bytes> wrapper to the supplied value, if
 * - We don't already have a Bytes wrapper
 * - The message is not an Ethereum-style message
 */
function u8aWrapBytes(bytes) {
    const u8a = (0, toU8a_js_1.u8aToU8a)(bytes);
    return u8aIsWrapped(u8a, true)
        ? u8a
        : (0, concat_js_1.u8aConcatStrict)([exports.U8A_WRAP_PREFIX, u8a, exports.U8A_WRAP_POSTFIX]);
}
