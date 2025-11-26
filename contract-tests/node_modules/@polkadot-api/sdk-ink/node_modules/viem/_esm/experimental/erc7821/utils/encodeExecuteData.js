import { encodeFunctionData, } from '../../../utils/abi/encodeFunctionData.js';
import { abi, executionMode } from '../constants.js';
import { encodeCalls } from './encodeCalls.js';
export function encodeExecuteData(parameters) {
    const { calls, opData } = parameters;
    const encodedCalls = encodeCalls(calls, opData);
    const mode = opData ? executionMode.opData : executionMode.default;
    return encodeFunctionData({
        abi,
        functionName: 'execute',
        args: [mode, encodedCalls],
    });
}
//# sourceMappingURL=encodeExecuteData.js.map