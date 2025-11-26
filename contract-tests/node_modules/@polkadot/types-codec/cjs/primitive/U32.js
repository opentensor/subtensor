"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u32 = void 0;
const UInt_js_1 = require("../base/UInt.js");
/**
 * @name u32
 * @description
 * A 32-bit unsigned integer
 */
class u32 extends UInt_js_1.UInt.with(32) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u32';
}
exports.u32 = u32;
