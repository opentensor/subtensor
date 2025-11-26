"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.i256 = void 0;
const Int_js_1 = require("../base/Int.js");
/**
 * @name i256
 * @description
 * A 256-bit signed integer
 */
class i256 extends Int_js_1.Int.with(256) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i256';
}
exports.i256 = i256;
