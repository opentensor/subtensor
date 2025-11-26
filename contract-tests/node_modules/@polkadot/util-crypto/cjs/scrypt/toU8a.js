"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.scryptToU8a = scryptToU8a;
const util_1 = require("@polkadot/util");
const bn_js_1 = require("../bn.js");
function scryptToU8a(salt, { N, p, r }) {
    return (0, util_1.u8aConcat)(salt, (0, util_1.bnToU8a)(N, bn_js_1.BN_LE_32_OPTS), (0, util_1.bnToU8a)(p, bn_js_1.BN_LE_32_OPTS), (0, util_1.bnToU8a)(r, bn_js_1.BN_LE_32_OPTS));
}
