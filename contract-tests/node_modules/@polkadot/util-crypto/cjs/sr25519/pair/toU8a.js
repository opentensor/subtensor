"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sr25519KeypairToU8a = sr25519KeypairToU8a;
const util_1 = require("@polkadot/util");
function sr25519KeypairToU8a({ publicKey, secretKey }) {
    return (0, util_1.u8aConcat)(secretKey, publicKey).slice();
}
