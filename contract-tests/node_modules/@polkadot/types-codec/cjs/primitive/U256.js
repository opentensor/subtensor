"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u256 = void 0;
const UInt_js_1 = require("../base/UInt.js");
/**
 * @name u256
 * @description
 * A 256-bit unsigned integer
 */
class u256 extends UInt_js_1.UInt.with(256) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u256';
}
exports.u256 = u256;
