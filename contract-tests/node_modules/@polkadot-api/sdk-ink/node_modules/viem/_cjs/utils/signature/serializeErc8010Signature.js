"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.serializeErc8010Signature = serializeErc8010Signature;
const erc8010_1 = require("ox/erc8010");
const toBytes_js_1 = require("../encoding/toBytes.js");
function serializeErc8010Signature(parameters) {
    const { address, data, signature, to = 'hex' } = parameters;
    const signature_ = erc8010_1.SignatureErc8010.wrap({
        authorization: {
            address: parameters.authorization.address,
            chainId: parameters.authorization.chainId,
            nonce: BigInt(parameters.authorization.nonce),
            r: BigInt(parameters.authorization.r),
            s: BigInt(parameters.authorization.s),
            yParity: parameters.authorization.yParity,
        },
        data,
        signature,
        to: address,
    });
    if (to === 'hex')
        return signature_;
    return (0, toBytes_js_1.hexToBytes)(signature_);
}
//# sourceMappingURL=serializeErc8010Signature.js.map