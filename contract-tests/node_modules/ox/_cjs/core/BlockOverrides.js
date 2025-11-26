"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fromRpc = fromRpc;
exports.toRpc = toRpc;
const Hex = require("./Hex.js");
const Withdrawal = require("./Withdrawal.js");
function fromRpc(rpcBlockOverrides) {
    return {
        ...(rpcBlockOverrides.baseFeePerGas && {
            baseFeePerGas: BigInt(rpcBlockOverrides.baseFeePerGas),
        }),
        ...(rpcBlockOverrides.blobBaseFee && {
            blobBaseFee: BigInt(rpcBlockOverrides.blobBaseFee),
        }),
        ...(rpcBlockOverrides.feeRecipient && {
            feeRecipient: rpcBlockOverrides.feeRecipient,
        }),
        ...(rpcBlockOverrides.gasLimit && {
            gasLimit: BigInt(rpcBlockOverrides.gasLimit),
        }),
        ...(rpcBlockOverrides.number && {
            number: BigInt(rpcBlockOverrides.number),
        }),
        ...(rpcBlockOverrides.prevRandao && {
            prevRandao: BigInt(rpcBlockOverrides.prevRandao),
        }),
        ...(rpcBlockOverrides.time && {
            time: BigInt(rpcBlockOverrides.time),
        }),
        ...(rpcBlockOverrides.withdrawals && {
            withdrawals: rpcBlockOverrides.withdrawals.map(Withdrawal.fromRpc),
        }),
    };
}
function toRpc(blockOverrides) {
    return {
        ...(typeof blockOverrides.baseFeePerGas === 'bigint' && {
            baseFeePerGas: Hex.fromNumber(blockOverrides.baseFeePerGas),
        }),
        ...(typeof blockOverrides.blobBaseFee === 'bigint' && {
            blobBaseFee: Hex.fromNumber(blockOverrides.blobBaseFee),
        }),
        ...(typeof blockOverrides.feeRecipient === 'string' && {
            feeRecipient: blockOverrides.feeRecipient,
        }),
        ...(typeof blockOverrides.gasLimit === 'bigint' && {
            gasLimit: Hex.fromNumber(blockOverrides.gasLimit),
        }),
        ...(typeof blockOverrides.number === 'bigint' && {
            number: Hex.fromNumber(blockOverrides.number),
        }),
        ...(typeof blockOverrides.prevRandao === 'bigint' && {
            prevRandao: Hex.fromNumber(blockOverrides.prevRandao),
        }),
        ...(typeof blockOverrides.time === 'bigint' && {
            time: Hex.fromNumber(blockOverrides.time),
        }),
        ...(blockOverrides.withdrawals && {
            withdrawals: blockOverrides.withdrawals.map(Withdrawal.toRpc),
        }),
    };
}
//# sourceMappingURL=BlockOverrides.js.map