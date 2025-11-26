"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.f64 = void 0;
const Float_js_1 = require("../native/Float.js");
/**
 * @name f64
 * @description
 * A 64-bit float
 */
class f64 extends Float_js_1.Float.with(64) {
    // NOTE without this, we cannot properly determine extensions
    __FloatType = 'f64';
}
exports.f64 = f64;
