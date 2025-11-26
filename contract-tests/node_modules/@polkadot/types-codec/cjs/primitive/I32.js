"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.i32 = void 0;
const Int_js_1 = require("../base/Int.js");
/**
 * @name i32
 * @description
 * A 32-bit signed integer
 */
class i32 extends Int_js_1.Int.with(32) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i32';
}
exports.i32 = i32;
