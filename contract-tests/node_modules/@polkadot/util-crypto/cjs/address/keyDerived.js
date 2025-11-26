"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createKeyDerived = createKeyDerived;
const util_1 = require("@polkadot/util");
const asU8a_js_1 = require("../blake2/asU8a.js");
const bn_js_1 = require("../bn.js");
const decode_js_1 = require("./decode.js");
const PREFIX = (0, util_1.stringToU8a)('modlpy/utilisuba');
function createKeyDerived(who, index) {
    return (0, asU8a_js_1.blake2AsU8a)((0, util_1.u8aConcat)(PREFIX, (0, decode_js_1.decodeAddress)(who), (0, util_1.bnToU8a)(index, bn_js_1.BN_LE_16_OPTS)));
}
