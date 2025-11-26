import * as AbiParameters from 'ox/AbiParameters';
import { encodeFunctionData, } from '../../../utils/abi/encodeFunctionData.js';
import { abi, executionMode } from '../constants.js';
import { encodeCalls } from './encodeCalls.js';
export function encodeExecuteBatchesData(parameters) {
    const { batches } = parameters;
    const encodedBatches = AbiParameters.encode(AbiParameters.from('bytes[]'), [
        batches.map((b) => {
            const batch = b;
            return encodeCalls(batch.calls, batch.opData);
        }),
    ]);
    return encodeFunctionData({
        abi,
        functionName: 'execute',
        args: [executionMode.batchOfBatches, encodedBatches],
    });
}
//# sourceMappingURL=encodeExecuteBatchesData.js.map