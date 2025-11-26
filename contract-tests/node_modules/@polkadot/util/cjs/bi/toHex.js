"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nToHex = nToHex;
const index_js_1 = require("../u8a/index.js");
const toU8a_js_1 = require("./toU8a.js");
/**
 * @name nToHex
 * @summary Creates a hex value from a bigint object.
 */
function nToHex(value, { bitLength = -1, isLe = false, isNegative = false } = {}) {
    return (0, index_js_1.u8aToHex)((0, toU8a_js_1.nToU8a)(value || 0, { bitLength, isLe, isNegative }));
}
