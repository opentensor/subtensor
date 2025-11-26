"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stringify = stringify;
const bigInt_js_1 = require("./is/bigInt.js");
/** @internal */
function replacer(_, v) {
    return (0, bigInt_js_1.isBigInt)(v)
        ? v.toString()
        : v;
}
/**
 * @name stringify
 * @summary Performs a JSON.stringify, with BigInt handling
 * @description A wrapper for JSON.stringify that handles BigInt values transparently, converting them to string. No differences from the native JSON.stringify function otherwise.
 */
function stringify(value, space) {
    return JSON.stringify(value, replacer, space);
}
