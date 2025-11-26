"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sign = sign;
exports.signGeneral = signGeneral;
const util_crypto_1 = require("@polkadot/util-crypto");
function sign(_registry, signerPair, u8a, options) {
    const encoded = u8a.length > 256
        ? (0, util_crypto_1.blake2AsU8a)(u8a)
        : u8a;
    return signerPair.sign(encoded, options);
}
function signGeneral(registry, u8a) {
    const encoded = registry.hash(u8a);
    return encoded;
}
