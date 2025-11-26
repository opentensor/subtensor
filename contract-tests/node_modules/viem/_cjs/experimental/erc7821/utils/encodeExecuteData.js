"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeExecuteData = encodeExecuteData;
const encodeFunctionData_js_1 = require("../../../utils/abi/encodeFunctionData.js");
const constants_js_1 = require("../constants.js");
const encodeCalls_js_1 = require("./encodeCalls.js");
function encodeExecuteData(parameters) {
    const { calls, opData } = parameters;
    const encodedCalls = (0, encodeCalls_js_1.encodeCalls)(calls, opData);
    const mode = opData ? constants_js_1.executionMode.opData : constants_js_1.executionMode.default;
    return (0, encodeFunctionData_js_1.encodeFunctionData)({
        abi: constants_js_1.abi,
        functionName: 'execute',
        args: [mode, encodedCalls],
    });
}
//# sourceMappingURL=encodeExecuteData.js.map