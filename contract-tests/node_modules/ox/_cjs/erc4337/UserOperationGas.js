"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fromRpc = fromRpc;
exports.toRpc = toRpc;
const Hex = require("../core/Hex.js");
function fromRpc(rpc) {
    return {
        ...rpc,
        callGasLimit: BigInt(rpc.callGasLimit),
        preVerificationGas: BigInt(rpc.preVerificationGas),
        verificationGasLimit: BigInt(rpc.verificationGasLimit),
        ...(rpc.paymasterVerificationGasLimit && {
            paymasterVerificationGasLimit: BigInt(rpc.paymasterVerificationGasLimit),
        }),
        ...(rpc.paymasterPostOpGasLimit && {
            paymasterPostOpGasLimit: BigInt(rpc.paymasterPostOpGasLimit),
        }),
    };
}
function toRpc(userOperationGas) {
    const rpc = {};
    rpc.callGasLimit = Hex.fromNumber(userOperationGas.callGasLimit);
    rpc.preVerificationGas = Hex.fromNumber(userOperationGas.preVerificationGas);
    rpc.verificationGasLimit = Hex.fromNumber(userOperationGas.verificationGasLimit);
    if (typeof userOperationGas.paymasterVerificationGasLimit === 'bigint')
        rpc.paymasterVerificationGasLimit = Hex.fromNumber(userOperationGas.paymasterVerificationGasLimit);
    if (typeof userOperationGas.paymasterPostOpGasLimit === 'bigint')
        rpc.paymasterPostOpGasLimit = Hex.fromNumber(userOperationGas.paymasterPostOpGasLimit);
    return rpc;
}
//# sourceMappingURL=UserOperationGas.js.map