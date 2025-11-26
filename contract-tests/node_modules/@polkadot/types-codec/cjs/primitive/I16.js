"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.i16 = void 0;
const Int_js_1 = require("../base/Int.js");
/**
 * @name i16
 * @description
 * A 16-bit signed integer
 */
class i16 extends Int_js_1.Int.with(16) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i16';
}
exports.i16 = i16;
