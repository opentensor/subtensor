"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isRiscV = isRiscV;
const eq_js_1 = require("../u8a/eq.js");
const u8a_js_1 = require("./u8a.js");
const ELF_MAGIC = new Uint8Array([0x7f, 0x45, 0x4c, 0x46]); // ELF magic bytes: 0x7f, 'E', 'L', 'F'
const PVM_MAGIC = new Uint8Array([0x50, 0x56, 0x4d, 0x00]); // 'P', 'V', 'M', 0x00
/**
 * @name isRiscV
 * @summary Tests if the input has a RISC-V header
 * @description
 * Checks to see if the input Uint8Array contains a valid RISC-V header
 */
function isRiscV(bytes) {
    if ((0, u8a_js_1.isU8a)(bytes)) {
        const start = bytes.subarray(0, 4);
        return (0, eq_js_1.u8aEq)(start, PVM_MAGIC) || (0, eq_js_1.u8aEq)(start, ELF_MAGIC);
    }
    return false;
}
