"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hashTypedData = hashTypedData;
const hashTypedData_js_1 = require("../../../utils/signature/hashTypedData.js");
function hashTypedData(parameters) {
    const { domain, message, primaryType, types, verifierDomain } = parameters;
    return (0, hashTypedData_js_1.hashTypedData)({
        domain: domain,
        types: {
            ...types,
            TypedDataSign: [
                { name: 'contents', type: primaryType },
                { name: 'name', type: 'string' },
                { name: 'version', type: 'string' },
                { name: 'chainId', type: 'uint256' },
                { name: 'verifyingContract', type: 'address' },
                { name: 'salt', type: 'bytes32' },
            ],
        },
        primaryType: 'TypedDataSign',
        message: {
            contents: message,
            ...verifierDomain,
        },
    });
}
//# sourceMappingURL=hashTypedData.js.map