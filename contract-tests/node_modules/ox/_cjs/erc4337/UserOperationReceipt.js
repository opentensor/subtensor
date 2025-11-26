"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fromRpc = fromRpc;
exports.toRpc = toRpc;
const Hex = require("../core/Hex.js");
const Log = require("../core/Log.js");
const TransactionReceipt = require("../core/TransactionReceipt.js");
function fromRpc(rpc) {
    return {
        ...rpc,
        actualGasCost: BigInt(rpc.actualGasCost),
        actualGasUsed: BigInt(rpc.actualGasUsed),
        logs: rpc.logs.map((log) => Log.fromRpc(log)),
        nonce: BigInt(rpc.nonce),
        receipt: TransactionReceipt.fromRpc(rpc.receipt),
    };
}
function toRpc(userOperationReceipt) {
    const rpc = {};
    rpc.actualGasCost = Hex.fromNumber(userOperationReceipt.actualGasCost);
    rpc.actualGasUsed = Hex.fromNumber(userOperationReceipt.actualGasUsed);
    rpc.entryPoint = userOperationReceipt.entryPoint;
    rpc.logs = userOperationReceipt.logs.map((log) => Log.toRpc(log));
    rpc.nonce = Hex.fromNumber(userOperationReceipt.nonce);
    rpc.receipt = TransactionReceipt.toRpc(userOperationReceipt.receipt);
    rpc.sender = userOperationReceipt.sender;
    rpc.success = userOperationReceipt.success;
    rpc.userOpHash = userOperationReceipt.userOpHash;
    if (userOperationReceipt.paymaster)
        rpc.paymaster = userOperationReceipt.paymaster;
    if (userOperationReceipt.reason)
        rpc.reason = userOperationReceipt.reason;
    return rpc;
}
//# sourceMappingURL=UserOperationReceipt.js.map