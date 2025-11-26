"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseErc8010Signature = parseErc8010Signature;
const erc8010_1 = require("ox/erc8010");
const toHex_js_1 = require("../encoding/toHex.js");
const isErc8010Signature_js_1 = require("./isErc8010Signature.js");
function parseErc8010Signature(signature) {
    if (!(0, isErc8010Signature_js_1.isErc8010Signature)(signature))
        return { signature };
    const { authorization: authorization_ox, to, ...rest } = erc8010_1.SignatureErc8010.unwrap(signature);
    return {
        authorization: {
            address: authorization_ox.address,
            chainId: authorization_ox.chainId,
            nonce: Number(authorization_ox.nonce),
            r: (0, toHex_js_1.numberToHex)(authorization_ox.r, { size: 32 }),
            s: (0, toHex_js_1.numberToHex)(authorization_ox.s, { size: 32 }),
            yParity: authorization_ox.yParity,
        },
        ...(to ? { address: to } : {}),
        ...rest,
    };
}
//# sourceMappingURL=parseErc8010Signature.js.map