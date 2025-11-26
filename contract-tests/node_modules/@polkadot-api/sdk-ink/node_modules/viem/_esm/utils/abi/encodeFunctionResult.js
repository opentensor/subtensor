import { AbiFunctionNotFoundError, AbiFunctionOutputsNotFoundError, InvalidArrayError, } from '../../errors/abi.js';
import { encodeAbiParameters, } from './encodeAbiParameters.js';
import { getAbiItem } from './getAbiItem.js';
const docsPath = '/docs/contract/encodeFunctionResult';
export function encodeFunctionResult(parameters) {
    const { abi, functionName, result } = parameters;
    let abiItem = abi[0];
    if (functionName) {
        const item = getAbiItem({ abi, name: functionName });
        if (!item)
            throw new AbiFunctionNotFoundError(functionName, { docsPath });
        abiItem = item;
    }
    if (abiItem.type !== 'function')
        throw new AbiFunctionNotFoundError(undefined, { docsPath });
    if (!abiItem.outputs)
        throw new AbiFunctionOutputsNotFoundError(abiItem.name, { docsPath });
    const values = (() => {
        if (abiItem.outputs.length === 0)
            return [];
        if (abiItem.outputs.length === 1)
            return [result];
        if (Array.isArray(result))
            return result;
        throw new InvalidArrayError(result);
    })();
    return encodeAbiParameters(abiItem.outputs, values);
}
//# sourceMappingURL=encodeFunctionResult.js.map