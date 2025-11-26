import { batchGatewayAbi } from '../../constants/abis.js';
import { solidityError } from '../../constants/solidity.js';
import { decodeFunctionData } from '../abi/decodeFunctionData.js';
import { encodeErrorResult } from '../abi/encodeErrorResult.js';
import { encodeFunctionResult } from '../abi/encodeFunctionResult.js';
export const localBatchGatewayUrl = 'x-batch-gateway:true';
export async function localBatchGatewayRequest(parameters) {
    const { data, ccipRequest } = parameters;
    const { args: [queries], } = decodeFunctionData({ abi: batchGatewayAbi, data });
    const failures = [];
    const responses = [];
    await Promise.all(queries.map(async (query, i) => {
        try {
            responses[i] = query.urls.includes(localBatchGatewayUrl)
                ? await localBatchGatewayRequest({ data: query.data, ccipRequest })
                : await ccipRequest(query);
            failures[i] = false;
        }
        catch (err) {
            failures[i] = true;
            responses[i] = encodeError(err);
        }
    }));
    return encodeFunctionResult({
        abi: batchGatewayAbi,
        functionName: 'query',
        result: [failures, responses],
    });
}
function encodeError(error) {
    if (error.name === 'HttpRequestError' && error.status)
        return encodeErrorResult({
            abi: batchGatewayAbi,
            errorName: 'HttpError',
            args: [error.status, error.shortMessage],
        });
    return encodeErrorResult({
        abi: [solidityError],
        errorName: 'Error',
        args: ['shortMessage' in error ? error.shortMessage : error.message],
    });
}
//# sourceMappingURL=localBatchGatewayRequest.js.map