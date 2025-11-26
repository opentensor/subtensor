"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getUserOperationTypedData = getUserOperationTypedData;
const toPackedUserOperation_js_1 = require("./toPackedUserOperation.js");
const types = {
    PackedUserOperation: [
        { type: 'address', name: 'sender' },
        { type: 'uint256', name: 'nonce' },
        { type: 'bytes', name: 'initCode' },
        { type: 'bytes', name: 'callData' },
        { type: 'bytes32', name: 'accountGasLimits' },
        { type: 'uint256', name: 'preVerificationGas' },
        { type: 'bytes32', name: 'gasFees' },
        { type: 'bytes', name: 'paymasterAndData' },
    ],
};
function getUserOperationTypedData(parameters) {
    const { chainId, entryPointAddress, userOperation } = parameters;
    const packedUserOp = (0, toPackedUserOperation_js_1.toPackedUserOperation)(userOperation);
    return {
        types,
        primaryType: 'PackedUserOperation',
        domain: {
            name: 'ERC4337',
            version: '1',
            chainId,
            verifyingContract: entryPointAddress,
        },
        message: packedUserOp,
    };
}
//# sourceMappingURL=getUserOperationTypedData.js.map