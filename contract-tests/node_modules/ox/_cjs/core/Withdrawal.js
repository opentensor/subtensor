"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fromRpc = fromRpc;
exports.toRpc = toRpc;
const Hex = require("./Hex.js");
function fromRpc(withdrawal) {
    return {
        ...withdrawal,
        amount: BigInt(withdrawal.amount),
        index: Number(withdrawal.index),
        validatorIndex: Number(withdrawal.validatorIndex),
    };
}
function toRpc(withdrawal) {
    return {
        address: withdrawal.address,
        amount: Hex.fromNumber(withdrawal.amount),
        index: Hex.fromNumber(withdrawal.index),
        validatorIndex: Hex.fromNumber(withdrawal.validatorIndex),
    };
}
//# sourceMappingURL=Withdrawal.js.map