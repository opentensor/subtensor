"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fallbackTransactionErrorMagicIdentifier = exports.fallbackMagicIdentifier = void 0;
exports.sendCalls = sendCalls;
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const base_js_1 = require("../../errors/base.js");
const rpc_js_1 = require("../../errors/rpc.js");
const encodeFunctionData_js_1 = require("../../utils/abi/encodeFunctionData.js");
const concat_js_1 = require("../../utils/data/concat.js");
const fromHex_js_1 = require("../../utils/encoding/fromHex.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
const getTransactionError_js_1 = require("../../utils/errors/getTransactionError.js");
const sendTransaction_js_1 = require("./sendTransaction.js");
exports.fallbackMagicIdentifier = '0x5792579257925792579257925792579257925792579257925792579257925792';
exports.fallbackTransactionErrorMagicIdentifier = (0, toHex_js_1.numberToHex)(0, {
    size: 32,
});
async function sendCalls(client, parameters) {
    const { account: account_ = client.account, capabilities, chain = client.chain, experimental_fallback, experimental_fallbackDelay = 32, forceAtomic = false, id, version = '2.0.0', } = parameters;
    const account = account_ ? (0, parseAccount_js_1.parseAccount)(account_) : null;
    const calls = parameters.calls.map((call_) => {
        const call = call_;
        const data = call.abi
            ? (0, encodeFunctionData_js_1.encodeFunctionData)({
                abi: call.abi,
                functionName: call.functionName,
                args: call.args,
            })
            : call.data;
        return {
            data: call.dataSuffix && data ? (0, concat_js_1.concat)([data, call.dataSuffix]) : data,
            to: call.to,
            value: call.value ? (0, toHex_js_1.numberToHex)(call.value) : undefined,
        };
    });
    try {
        const response = await client.request({
            method: 'wallet_sendCalls',
            params: [
                {
                    atomicRequired: forceAtomic,
                    calls,
                    capabilities,
                    chainId: (0, toHex_js_1.numberToHex)(chain.id),
                    from: account?.address,
                    id,
                    version,
                },
            ],
        }, { retryCount: 0 });
        if (typeof response === 'string')
            return { id: response };
        return response;
    }
    catch (err) {
        const error = err;
        if (experimental_fallback &&
            (error.name === 'MethodNotFoundRpcError' ||
                error.name === 'MethodNotSupportedRpcError' ||
                error.name === 'UnknownRpcError' ||
                error.details
                    .toLowerCase()
                    .includes('does not exist / is not available') ||
                error.details.toLowerCase().includes('missing or invalid. request()') ||
                error.details
                    .toLowerCase()
                    .includes('did not match any variant of untagged enum') ||
                error.details
                    .toLowerCase()
                    .includes('account upgraded to unsupported contract') ||
                error.details.toLowerCase().includes('eip-7702 not supported') ||
                error.details.toLowerCase().includes('unsupported wc_ method') ||
                error.details
                    .toLowerCase()
                    .includes('feature toggled misconfigured') ||
                error.details
                    .toLowerCase()
                    .includes('jsonrpcengine: response has no error or result for request'))) {
            if (capabilities) {
                const hasNonOptionalCapability = Object.values(capabilities).some((capability) => !capability.optional);
                if (hasNonOptionalCapability) {
                    const message = 'non-optional `capabilities` are not supported on fallback to `eth_sendTransaction`.';
                    throw new rpc_js_1.UnsupportedNonOptionalCapabilityError(new base_js_1.BaseError(message, {
                        details: message,
                    }));
                }
            }
            if (forceAtomic && calls.length > 1) {
                const message = '`forceAtomic` is not supported on fallback to `eth_sendTransaction`.';
                throw new rpc_js_1.AtomicityNotSupportedError(new base_js_1.BaseError(message, {
                    details: message,
                }));
            }
            const promises = [];
            for (const call of calls) {
                const promise = (0, sendTransaction_js_1.sendTransaction)(client, {
                    account,
                    chain,
                    data: call.data,
                    to: call.to,
                    value: call.value ? (0, fromHex_js_1.hexToBigInt)(call.value) : undefined,
                });
                promises.push(promise);
                if (experimental_fallbackDelay > 0)
                    await new Promise((resolve) => setTimeout(resolve, experimental_fallbackDelay));
            }
            const results = await Promise.allSettled(promises);
            if (results.every((r) => r.status === 'rejected'))
                throw results[0].reason;
            const hashes = results.map((result) => {
                if (result.status === 'fulfilled')
                    return result.value;
                return exports.fallbackTransactionErrorMagicIdentifier;
            });
            return {
                id: (0, concat_js_1.concat)([
                    ...hashes,
                    (0, toHex_js_1.numberToHex)(chain.id, { size: 32 }),
                    exports.fallbackMagicIdentifier,
                ]),
            };
        }
        throw (0, getTransactionError_js_1.getTransactionError)(err, {
            ...parameters,
            account,
            chain: parameters.chain,
        });
    }
}
//# sourceMappingURL=sendCalls.js.map