"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createKeyMulti = createKeyMulti;
const util_1 = require("@polkadot/util");
const asU8a_js_1 = require("../blake2/asU8a.js");
const bn_js_1 = require("../bn.js");
const util_js_1 = require("./util.js");
const PREFIX = (0, util_1.stringToU8a)('modlpy/utilisuba');
function createKeyMulti(who, threshold) {
    return (0, asU8a_js_1.blake2AsU8a)((0, util_1.u8aConcat)(PREFIX, (0, util_1.compactToU8a)(who.length), ...(0, util_1.u8aSorted)(who.map(util_js_1.addressToU8a)), (0, util_1.bnToU8a)(threshold, bn_js_1.BN_LE_16_OPTS)));
}
