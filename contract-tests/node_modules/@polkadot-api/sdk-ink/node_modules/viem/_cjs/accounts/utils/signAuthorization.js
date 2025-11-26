"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.signAuthorization = signAuthorization;
const hashAuthorization_js_1 = require("../../utils/authorization/hashAuthorization.js");
const sign_js_1 = require("./sign.js");
async function signAuthorization(parameters) {
    const { chainId, nonce, privateKey, to = 'object' } = parameters;
    const address = parameters.contractAddress ?? parameters.address;
    const signature = await (0, sign_js_1.sign)({
        hash: (0, hashAuthorization_js_1.hashAuthorization)({ address, chainId, nonce }),
        privateKey,
        to,
    });
    if (to === 'object')
        return {
            address,
            chainId,
            nonce,
            ...signature,
        };
    return signature;
}
//# sourceMappingURL=signAuthorization.js.map