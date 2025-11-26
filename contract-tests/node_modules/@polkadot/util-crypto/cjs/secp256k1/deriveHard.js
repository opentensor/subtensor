"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1DeriveHard = secp256k1DeriveHard;
const util_1 = require("@polkadot/util");
const asU8a_js_1 = require("../blake2/asU8a.js");
const HDKD = (0, util_1.compactAddLength)((0, util_1.stringToU8a)('Secp256k1HDKD'));
function secp256k1DeriveHard(seed, chainCode) {
    if (!(0, util_1.isU8a)(chainCode) || chainCode.length !== 32) {
        throw new Error('Invalid chainCode passed to derive');
    }
    // NOTE This is specific to the Substrate HDD derivation, so always use the blake2 hasher
    return (0, asU8a_js_1.blake2AsU8a)((0, util_1.u8aConcat)(HDKD, seed, chainCode), 256);
}
