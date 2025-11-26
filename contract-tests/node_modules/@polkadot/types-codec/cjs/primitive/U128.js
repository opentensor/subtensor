"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u128 = void 0;
const UInt_js_1 = require("../base/UInt.js");
/**
 * @name u128
 * @description
 * A 128-bit unsigned integer
 */
class u128 extends UInt_js_1.UInt.with(128) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u128';
}
exports.u128 = u128;
