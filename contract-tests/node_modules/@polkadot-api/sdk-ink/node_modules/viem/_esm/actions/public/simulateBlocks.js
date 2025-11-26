import * as BlockOverrides from 'ox/BlockOverrides';
import { parseAccount, } from '../../accounts/utils/parseAccount.js';
import { AbiDecodingZeroDataError } from '../../errors/abi.js';
import { RawContractError } from '../../errors/contract.js';
import { UnknownNodeError } from '../../errors/node.js';
import { decodeFunctionResult, } from '../../utils/abi/decodeFunctionResult.js';
import { encodeFunctionData, } from '../../utils/abi/encodeFunctionData.js';
import { concat } from '../../utils/data/concat.js';
import { numberToHex, } from '../../utils/encoding/toHex.js';
import { getContractError } from '../../utils/errors/getContractError.js';
import { getNodeError, } from '../../utils/errors/getNodeError.js';
import { formatBlock, } from '../../utils/formatters/block.js';
import { formatLog } from '../../utils/formatters/log.js';
import { formatTransactionRequest, } from '../../utils/formatters/transactionRequest.js';
import { serializeStateOverride, } from '../../utils/stateOverride.js';
import { assertRequest, } from '../../utils/transaction/assertRequest.js';
/**
 * Simulates a set of calls on block(s) with optional block and state overrides.
 *
 * @example
 * ```ts
 * import { createClient, http, parseEther } from 'viem'
 * import { simulate } from 'viem/actions'
 * import { mainnet } from 'viem/chains'
 *
 * const client = createClient({
 *   chain: mainnet,
 *   transport: http(),
 * })
 *
 * const result = await simulate(client, {
 *   blocks: [{
 *     blockOverrides: {
 *       number: 69420n,
 *     },
 *     calls: [{
 *       {
 *         account: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *         data: '0xdeadbeef',
 *         to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *       },
 *       {
 *         account: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *         to: '0x70997970c51812dc3a010c7d01b50e0d17dc79c8',
 *         value: parseEther('1'),
 *       },
 *     }],
 *     stateOverrides: [{
 *       address: '0x5a0b54d5dc17e482fe8b0bdca5320161b95fb929',
 *       balance: parseEther('10'),
 *     }],
 *   }]
 * })
 * ```
 *
 * @param client - Client to use.
 * @param parameters - {@link SimulateBlocksParameters}
 * @returns Simulated blocks. {@link SimulateBlocksReturnType}
 */
export async function simulateBlocks(client, parameters) {
    const { blockNumber, blockTag = client.experimental_blockTag ?? 'latest', blocks, returnFullTransactions, traceTransfers, validation, } = parameters;
    try {
        const blockStateCalls = [];
        for (const block of blocks) {
            const blockOverrides = block.blockOverrides
                ? BlockOverrides.toRpc(block.blockOverrides)
                : undefined;
            const calls = block.calls.map((call_) => {
                const call = call_;
                const account = call.account ? parseAccount(call.account) : undefined;
                const data = call.abi ? encodeFunctionData(call) : call.data;
                const request = {
                    ...call,
                    account,
                    data: call.dataSuffix
                        ? concat([data || '0x', call.dataSuffix])
                        : data,
                    from: call.from ?? account?.address,
                };
                assertRequest(request);
                return formatTransactionRequest(request);
            });
            const stateOverrides = block.stateOverrides
                ? serializeStateOverride(block.stateOverrides)
                : undefined;
            blockStateCalls.push({
                blockOverrides,
                calls,
                stateOverrides,
            });
        }
        const blockNumberHex = typeof blockNumber === 'bigint' ? numberToHex(blockNumber) : undefined;
        const block = blockNumberHex || blockTag;
        const result = await client.request({
            method: 'eth_simulateV1',
            params: [
                { blockStateCalls, returnFullTransactions, traceTransfers, validation },
                block,
            ],
        });
        return result.map((block, i) => ({
            ...formatBlock(block),
            calls: block.calls.map((call, j) => {
                const { abi, args, functionName, to } = blocks[i].calls[j];
                const data = call.error?.data ?? call.returnData;
                const gasUsed = BigInt(call.gasUsed);
                const logs = call.logs?.map((log) => formatLog(log));
                const status = call.status === '0x1' ? 'success' : 'failure';
                const result = abi && status === 'success' && data !== '0x'
                    ? decodeFunctionResult({
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
                        error = new AbiDecodingZeroDataError();
                    else if (call.error)
                        error = new RawContractError(call.error);
                    if (!error)
                        return undefined;
                    return getContractError(error, {
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
        const error = getNodeError(cause, {});
        if (error instanceof UnknownNodeError)
            throw cause;
        throw error;
    }
}
//# sourceMappingURL=simulateBlocks.js.map