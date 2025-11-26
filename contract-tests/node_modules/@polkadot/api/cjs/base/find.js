"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.findCall = findCall;
exports.findError = findError;
const util_1 = require("@polkadot/util");
function findCall(registry, callIndex) {
    return registry.findMetaCall((0, util_1.u8aToU8a)(callIndex));
}
function findError(registry, errorIndex) {
    return registry.findMetaError((0, util_1.u8aToU8a)(errorIndex));
}
