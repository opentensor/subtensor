"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.i128 = void 0;
const Int_js_1 = require("../base/Int.js");
/**
 * @name i128
 * @description
 * A 128-bit signed integer
 */
class i128 extends Int_js_1.Int.with(128) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i128';
}
exports.i128 = i128;
