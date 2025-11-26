"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getUserOperationHash = getUserOperationHash;
const encodeAbiParameters_js_1 = require("../../../utils/abi/encodeAbiParameters.js");
const keccak256_js_1 = require("../../../utils/hash/keccak256.js");
const hashTypedData_js_1 = require("../../../utils/signature/hashTypedData.js");
const getInitCode_js_1 = require("./getInitCode.js");
const getUserOperationTypedData_js_1 = require("./getUserOperationTypedData.js");
const toPackedUserOperation_js_1 = require("./toPackedUserOperation.js");
function getUserOperationHash(parameters) {
    const { chainId, entryPointAddress, entryPointVersion } = parameters;
    const userOperation = parameters.userOperation;
    const { authorization, callData = '0x', callGasLimit, maxFeePerGas, maxPriorityFeePerGas, nonce, paymasterAndData = '0x', preVerificationGas, sender, verificationGasLimit, } = userOperation;
    if (entryPointVersion === '0.8')
        return (0, hashTypedData_js_1.hashTypedData)((0, getUserOperationTypedData_js_1.getUserOperationTypedData)({
            chainId,
            entryPointAddress,
            userOperation,
        }));
    const packedUserOp = (() => {
        if (entryPointVersion === '0.6') {
            const factory = userOperation.initCode?.slice(0, 42);
            const factoryData = userOperation.initCode?.slice(42);
            const initCode = (0, getInitCode_js_1.getInitCode)({
                authorization,
                factory,
                factoryData,
            });
            return (0, encodeAbiParameters_js_1.encodeAbiParameters)([
                { type: 'address' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'uint256' },
                { type: 'bytes32' },
            ], [
                sender,
                nonce,
                (0, keccak256_js_1.keccak256)(initCode),
                (0, keccak256_js_1.keccak256)(callData),
                callGasLimit,
                verificationGasLimit,
                preVerificationGas,
                maxFeePerGas,
                maxPriorityFeePerGas,
                (0, keccak256_js_1.keccak256)(paymasterAndData),
            ]);
        }
        if (entryPointVersion === '0.7') {
            const packedUserOp = (0, toPackedUserOperation_js_1.toPackedUserOperation)(userOperation);
            return (0, encodeAbiParameters_js_1.encodeAbiParameters)([
                { type: 'address' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'bytes32' },
                { type: 'uint256' },
                { type: 'bytes32' },
                { type: 'bytes32' },
            ], [
                packedUserOp.sender,
                packedUserOp.nonce,
                (0, keccak256_js_1.keccak256)(packedUserOp.initCode),
                (0, keccak256_js_1.keccak256)(packedUserOp.callData),
                packedUserOp.accountGasLimits,
                packedUserOp.preVerificationGas,
                packedUserOp.gasFees,
                (0, keccak256_js_1.keccak256)(packedUserOp.paymasterAndData),
            ]);
        }
        throw new Error(`entryPointVersion "${entryPointVersion}" not supported.`);
    })();
    return (0, keccak256_js_1.keccak256)((0, encodeAbiParameters_js_1.encodeAbiParameters)([{ type: 'bytes32' }, { type: 'address' }, { type: 'uint256' }], [(0, keccak256_js_1.keccak256)(packedUserOp), entryPointAddress, BigInt(chainId)]));
}
//# sourceMappingURL=getUserOperationHash.js.map