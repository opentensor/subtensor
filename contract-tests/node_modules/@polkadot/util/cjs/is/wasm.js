"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isWasm = isWasm;
const eq_js_1 = require("../u8a/eq.js");
const u8a_js_1 = require("./u8a.js");
const WASM_MAGIC = new Uint8Array([0, 97, 115, 109]); // \0asm
/**
 * @name isWasm
 * @summary Tests if the input has a WASM header
 * @description
 * Checks to see if the input Uint8Array contains a valid WASM header
 */
function isWasm(value) {
    return (0, u8a_js_1.isU8a)(value) && (0, eq_js_1.u8aEq)(value.subarray(0, 4), WASM_MAGIC);
}
