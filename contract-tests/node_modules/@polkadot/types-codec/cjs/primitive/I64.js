"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.i64 = void 0;
const Int_js_1 = require("../base/Int.js");
/**
 * @name i64
 * @description
 * A 64-bit signed integer
 */
class i64 extends Int_js_1.Int.with(64) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i64';
}
exports.i64 = i64;
