"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.localBatchGatewayUrl = void 0;
exports.localBatchGatewayRequest = localBatchGatewayRequest;
const abis_js_1 = require("../../constants/abis.js");
const solidity_js_1 = require("../../constants/solidity.js");
const decodeFunctionData_js_1 = require("../abi/decodeFunctionData.js");
const encodeErrorResult_js_1 = require("../abi/encodeErrorResult.js");
const encodeFunctionResult_js_1 = require("../abi/encodeFunctionResult.js");
exports.localBatchGatewayUrl = 'x-batch-gateway:true';
async function localBatchGatewayRequest(parameters) {
    const { data, ccipRequest } = parameters;
    const { args: [queries], } = (0, decodeFunctionData_js_1.decodeFunctionData)({ abi: abis_js_1.batchGatewayAbi, data });
    const failures = [];
    const responses = [];
    await Promise.all(queries.map(async (query, i) => {
        try {
            responses[i] = query.urls.includes(exports.localBatchGatewayUrl)
                ? await localBatchGatewayRequest({ data: query.data, ccipRequest })
                : await ccipRequest(query);
            failures[i] = false;
        }
        catch (err) {
            failures[i] = true;
            responses[i] = encodeError(err);
        }
    }));
    return (0, encodeFunctionResult_js_1.encodeFunctionResult)({
        abi: abis_js_1.batchGatewayAbi,
        functionName: 'query',
        result: [failures, responses],
    });
}
function encodeError(error) {
    if (error.name === 'HttpRequestError' && error.status)
        return (0, encodeErrorResult_js_1.encodeErrorResult)({
            abi: abis_js_1.batchGatewayAbi,
            errorName: 'HttpError',
            args: [error.status, error.shortMessage],
        });
    return (0, encodeErrorResult_js_1.encodeErrorResult)({
        abi: [solidity_js_1.solidityError],
        errorName: 'Error',
        args: ['shortMessage' in error ? error.shortMessage : error.message],
    });
}
//# sourceMappingURL=localBatchGatewayRequest.js.map