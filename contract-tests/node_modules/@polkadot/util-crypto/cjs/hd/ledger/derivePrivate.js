"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ledgerDerivePrivate = ledgerDerivePrivate;
const util_1 = require("@polkadot/util");
const bn_js_1 = require("../../bn.js");
const index_js_1 = require("../../hmac/index.js");
function ledgerDerivePrivate(xprv, index) {
    const kl = xprv.subarray(0, 32);
    const kr = xprv.subarray(32, 64);
    const cc = xprv.subarray(64, 96);
    const data = (0, util_1.u8aConcat)([0], kl, kr, (0, util_1.bnToU8a)(index, bn_js_1.BN_LE_32_OPTS));
    const z = (0, index_js_1.hmacShaAsU8a)(cc, data, 512);
    data[0] = 0x01;
    return (0, util_1.u8aConcat)((0, util_1.bnToU8a)((0, util_1.u8aToBn)(kl, bn_js_1.BN_LE_OPTS).iadd((0, util_1.u8aToBn)(z.subarray(0, 28), bn_js_1.BN_LE_OPTS).imul(util_1.BN_EIGHT)), bn_js_1.BN_LE_512_OPTS).subarray(0, 32), (0, util_1.bnToU8a)((0, util_1.u8aToBn)(kr, bn_js_1.BN_LE_OPTS).iadd((0, util_1.u8aToBn)(z.subarray(32, 64), bn_js_1.BN_LE_OPTS)), bn_js_1.BN_LE_512_OPTS).subarray(0, 32), (0, index_js_1.hmacShaAsU8a)(cc, data, 512).subarray(32, 64));
}
