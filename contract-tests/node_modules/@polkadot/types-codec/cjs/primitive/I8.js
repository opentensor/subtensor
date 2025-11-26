"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.i8 = void 0;
const Int_js_1 = require("../base/Int.js");
/**
 * @name i8
 * @description
 * An 8-bit signed integer
 */
class i8 extends Int_js_1.Int.with(8) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i8';
}
exports.i8 = i8;
