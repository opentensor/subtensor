"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.simulateBlocks = simulateBlocks;
const BlockOverrides = require("ox/BlockOverrides");
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const abi_js_1 = require("../../errors/abi.js");
const contract_js_1 = require("../../errors/contract.js");
const node_js_1 = require("../../errors/node.js");
const decodeFunctionResult_js_1 = require("../../utils/abi/decodeFunctionResult.js");
const encodeFunctionData_js_1 = require("../../utils/abi/encodeFunctionData.js");
const concat_js_1 = require("../../utils/data/concat.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
const getContractError_js_1 = require("../../utils/errors/getContractError.js");
const getNodeError_js_1 = require("../../utils/errors/getNodeError.js");
const block_js_1 = require("../../utils/formatters/block.js");
const log_js_1 = require("../../utils/formatters/log.js");
const transactionRequest_js_1 = require("../../utils/formatters/transactionRequest.js");
const stateOverride_js_1 = require("../../utils/stateOverride.js");
const assertRequest_js_1 = require("../../utils/transaction/assertRequest.js");
async function simulateBlocks(client, parameters) {
    const { blockNumber, blockTag = client.experimental_blockTag ?? 'latest', blocks, returnFullTransactions, traceTransfers, validation, } = parameters;
    try {
        const blockStateCalls = [];
        for (const block of blocks) {
            const blockOverrides = block.blockOverrides
                ? BlockOverrides.toRpc(block.blockOverrides)
                : undefined;
            const calls = block.calls.map((call_) => {
                const call = call_;
                const account = call.account ? (0, parseAccount_js_1.parseAccount)(call.account) : undefined;
                const data = call.abi ? (0, encodeFunctionData_js_1.encodeFunctionData)(call) : call.data;
                const request = {
                    ...call,
                    account,
                    data: call.dataSuffix
                        ? (0, concat_js_1.concat)([data || '0x', call.dataSuffix])
                        : data,
                    from: call.from ?? account?.address,
                };
                (0, assertRequest_js_1.assertRequest)(request);
                return (0, transactionRequest_js_1.formatTransactionRequest)(request);
            });
            const stateOverrides = block.stateOverrides
                ? (0, stateOverride_js_1.serializeStateOverride)(block.stateOverrides)
                : undefined;
            blockStateCalls.push({
                blockOverrides,
                calls,
                stateOverrides,
            });
        }
        const blockNumberHex = typeof blockNumber === 'bigint' ? (0, toHex_js_1.numberToHex)(blockNumber) : undefined;
        const block = blockNumberHex || blockTag;
        const result = await client.request({
            method: 'eth_simulateV1',
            params: [
                { blockStateCalls, returnFullTransactions, traceTransfers, validation },
                block,
            ],
        });
        return result.map((block, i) => ({
            ...(0, block_js_1.formatBlock)(block),
            calls: block.calls.map((call, j) => {
                const { abi, args, functionName, to } = blocks[i].calls[j];
                const data = call.error?.data ?? call.returnData;
                const gasUsed = BigInt(call.gasUsed);
                const logs = call.logs?.map((log) => (0, log_js_1.formatLog)(log));
                const status = call.status === '0x1' ? 'success' : 'failure';
                const result = abi && status === 'success' && data !== '0x'
                    ? (0, decodeFunctionResult_js_1.decodeFunctionResult)({
                        abi,
                        data,
                        functionName,
                    })
                    : null;
                const error = (() => {
                    if (status === 'success')
                        return undefined;
                    let error;
                    if (call.error?.data === '0x')
                        error = new abi_js_1.AbiDecodingZeroDataError();
                    else if (call.error)
                        error = new contract_js_1.RawContractError(call.error);
                    if (!error)
                        return undefined;
                    return (0, getContractError_js_1.getContractError)(error, {
                        abi: (abi ?? []),
                        address: to ?? '0x',
                        args,
                        functionName: functionName ?? '<unknown>',
                    });
                })();
                return {
                    data,
                    gasUsed,
                    logs,
                    status,
                    ...(status === 'success'
                        ? {
                            result,
                        }
                        : {
                            error,
                        }),
                };
            }),
        }));
    }
    catch (e) {
        const cause = e;
        const error = (0, getNodeError_js_1.getNodeError)(cause, {});
        if (error instanceof node_js_1.UnknownNodeError)
            throw cause;
        throw error;
    }
}
//# sourceMappingURL=simulateBlocks.js.map