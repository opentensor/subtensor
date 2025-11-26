"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u8 = void 0;
const UInt_js_1 = require("../base/UInt.js");
/**
 * @name u8
 * @description
 * An 8-bit unsigned integer
 */
class u8 extends UInt_js_1.UInt.with(8) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u8';
}
exports.u8 = u8;
