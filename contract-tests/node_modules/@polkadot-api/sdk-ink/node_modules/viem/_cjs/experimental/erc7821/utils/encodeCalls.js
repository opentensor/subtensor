"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeCalls = encodeCalls;
const AbiParameters = require("ox/AbiParameters");
const encodeFunctionData_js_1 = require("../../../utils/abi/encodeFunctionData.js");
function encodeCalls(calls_, opData) {
    const calls = calls_.map((call_) => {
        const call = call_;
        return {
            data: call.abi ? (0, encodeFunctionData_js_1.encodeFunctionData)(call) : (call.data ?? '0x'),
            value: call.value ?? 0n,
            target: call.to,
        };
    });
    return AbiParameters.encode(AbiParameters.from([
        'struct Call { address target; uint256 value; bytes data; }',
        'Call[] calls',
        ...(opData ? ['bytes opData'] : []),
    ]), [calls, ...(opData ? [opData] : [])]);
}
//# sourceMappingURL=encodeCalls.js.map