"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeFunctionResult = encodeFunctionResult;
const abi_js_1 = require("../../errors/abi.js");
const encodeAbiParameters_js_1 = require("./encodeAbiParameters.js");
const getAbiItem_js_1 = require("./getAbiItem.js");
const docsPath = '/docs/contract/encodeFunctionResult';
function encodeFunctionResult(parameters) {
    const { abi, functionName, result } = parameters;
    let abiItem = abi[0];
    if (functionName) {
        const item = (0, getAbiItem_js_1.getAbiItem)({ abi, name: functionName });
        if (!item)
            throw new abi_js_1.AbiFunctionNotFoundError(functionName, { docsPath });
        abiItem = item;
    }
    if (abiItem.type !== 'function')
        throw new abi_js_1.AbiFunctionNotFoundError(undefined, { docsPath });
    if (!abiItem.outputs)
        throw new abi_js_1.AbiFunctionOutputsNotFoundError(abiItem.name, { docsPath });
    const values = (() => {
        if (abiItem.outputs.length === 0)
            return [];
        if (abiItem.outputs.length === 1)
            return [result];
        if (Array.isArray(result))
            return result;
        throw new abi_js_1.InvalidArrayError(result);
    })();
    return (0, encodeAbiParameters_js_1.encodeAbiParameters)(abiItem.outputs, values);
}
//# sourceMappingURL=encodeFunctionResult.js.map