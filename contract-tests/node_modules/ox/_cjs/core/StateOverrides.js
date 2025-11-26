"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fromRpc = fromRpc;
exports.toRpc = toRpc;
const Hex = require("./Hex.js");
function fromRpc(rpcStateOverrides) {
    const stateOverrides = {};
    for (const [address, accountOverridesRpc] of Object.entries(rpcStateOverrides)) {
        const accountOverrides = {};
        if (accountOverridesRpc.balance)
            accountOverrides.balance = BigInt(accountOverridesRpc.balance);
        if (accountOverridesRpc.code)
            accountOverrides.code = accountOverridesRpc.code;
        if (accountOverridesRpc.movePrecompileToAddress)
            accountOverrides.movePrecompileToAddress =
                accountOverridesRpc.movePrecompileToAddress;
        if (accountOverridesRpc.nonce)
            accountOverrides.nonce = BigInt(accountOverridesRpc.nonce);
        if (accountOverridesRpc.state)
            accountOverrides.state = accountOverridesRpc.state;
        if (accountOverridesRpc.stateDiff)
            accountOverrides.stateDiff = accountOverridesRpc.stateDiff;
        stateOverrides[address] = accountOverrides;
    }
    return stateOverrides;
}
function toRpc(stateOverrides) {
    const rpcStateOverrides = {};
    for (const [address, accountOverrides] of Object.entries(stateOverrides)) {
        const accountOverridesRpc = {};
        if (typeof accountOverrides.balance === 'bigint')
            accountOverridesRpc.balance = Hex.fromNumber(accountOverrides.balance);
        if (accountOverrides.code)
            accountOverridesRpc.code = accountOverrides.code;
        if (accountOverrides.movePrecompileToAddress)
            accountOverridesRpc.movePrecompileToAddress =
                accountOverrides.movePrecompileToAddress;
        if (typeof accountOverrides.nonce === 'bigint')
            accountOverridesRpc.nonce = Hex.fromNumber(accountOverrides.nonce);
        if (accountOverrides.state)
            accountOverridesRpc.state = accountOverrides.state;
        if (accountOverrides.stateDiff)
            accountOverridesRpc.stateDiff = accountOverrides.stateDiff;
        rpcStateOverrides[address] = accountOverridesRpc;
    }
    return rpcStateOverrides;
}
//# sourceMappingURL=StateOverrides.js.map