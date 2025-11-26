"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.u64 = void 0;
const UInt_js_1 = require("../base/UInt.js");
/**
 * @name u64
 * @description
 * A 64-bit unsigned integer
 */
class u64 extends UInt_js_1.UInt.with(64) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u64';
}
exports.u64 = u64;
