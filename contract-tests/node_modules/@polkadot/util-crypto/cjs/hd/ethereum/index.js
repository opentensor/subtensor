"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hdEthereum = hdEthereum;
const util_1 = require("@polkadot/util");
const bn_js_1 = require("../../bn.js");
const index_js_1 = require("../../hmac/index.js");
const index_js_2 = require("../../secp256k1/index.js");
const validatePath_js_1 = require("../validatePath.js");
const MASTER_SECRET = (0, util_1.stringToU8a)('Bitcoin seed');
function createCoded(secretKey, chainCode) {
    return {
        chainCode,
        publicKey: (0, index_js_2.secp256k1PairFromSeed)(secretKey).publicKey,
        secretKey
    };
}
function deriveChild(hd, index) {
    const indexBuffer = (0, util_1.bnToU8a)(index, bn_js_1.BN_BE_32_OPTS);
    const data = index >= validatePath_js_1.HARDENED
        ? (0, util_1.u8aConcat)(new Uint8Array(1), hd.secretKey, indexBuffer)
        : (0, util_1.u8aConcat)(hd.publicKey, indexBuffer);
    try {
        const I = (0, index_js_1.hmacShaAsU8a)(hd.chainCode, data, 512);
        return createCoded((0, index_js_2.secp256k1PrivateKeyTweakAdd)(hd.secretKey, I.slice(0, 32)), I.slice(32));
    }
    catch {
        // In case parse256(IL) >= n or ki == 0, proceed with the next value for i
        return deriveChild(hd, index + 1);
    }
}
function hdEthereum(seed, path = '') {
    const I = (0, index_js_1.hmacShaAsU8a)(MASTER_SECRET, seed, 512);
    let hd = createCoded(I.slice(0, 32), I.slice(32));
    if (!path || path === 'm' || path === 'M' || path === "m'" || path === "M'") {
        return hd;
    }
    if (!(0, validatePath_js_1.hdValidatePath)(path)) {
        throw new Error('Invalid derivation path');
    }
    const parts = path.split('/').slice(1);
    for (const p of parts) {
        hd = deriveChild(hd, parseInt(p, 10) + ((p.length > 1) && p.endsWith("'")
            ? validatePath_js_1.HARDENED
            : 0));
    }
    return hd;
}
