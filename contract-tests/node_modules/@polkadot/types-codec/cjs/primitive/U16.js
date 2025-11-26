"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u16 = void 0;
const UInt_js_1 = require("../base/UInt.js");
/**
 * @name u16
 * @description
 * A 16-bit unsigned integer
 */
class u16 extends UInt_js_1.UInt.with(16) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u16';
}
exports.u16 = u16;
