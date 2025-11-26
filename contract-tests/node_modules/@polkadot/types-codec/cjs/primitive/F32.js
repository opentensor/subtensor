"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.f32 = void 0;
const Float_js_1 = require("../native/Float.js");
/**
 * @name f32
 * @description
 * A 32-bit float
 */
class f32 extends Float_js_1.Float.with(32) {
    // NOTE without this, we cannot properly determine extensions
    __FloatType = 'f32';
}
exports.f32 = f32;
