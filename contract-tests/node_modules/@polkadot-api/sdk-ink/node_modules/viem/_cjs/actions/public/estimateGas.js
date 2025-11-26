"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.estimateGas = estimateGas;
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const base_js_1 = require("../../errors/base.js");
const recoverAuthorizationAddress_js_1 = require("../../utils/authorization/recoverAuthorizationAddress.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
const getEstimateGasError_js_1 = require("../../utils/errors/getEstimateGasError.js");
const extract_js_1 = require("../../utils/formatters/extract.js");
const transactionRequest_js_1 = require("../../utils/formatters/transactionRequest.js");
const stateOverride_js_1 = require("../../utils/stateOverride.js");
const assertRequest_js_1 = require("../../utils/transaction/assertRequest.js");
const prepareTransactionRequest_js_1 = require("../wallet/prepareTransactionRequest.js");
async function estimateGas(client, args) {
    const { account: account_ = client.account, prepare = true } = args;
    const account = account_ ? (0, parseAccount_js_1.parseAccount)(account_) : undefined;
    const parameters = (() => {
        if (Array.isArray(prepare))
            return prepare;
        if (account?.type !== 'local')
            return ['blobVersionedHashes'];
        return undefined;
    })();
    try {
        const { accessList, authorizationList, blobs, blobVersionedHashes, blockNumber, blockTag, data, gas, gasPrice, maxFeePerBlobGas, maxFeePerGas, maxPriorityFeePerGas, nonce, value, stateOverride, ...rest } = prepare
            ? (await (0, prepareTransactionRequest_js_1.prepareTransactionRequest)(client, {
                ...args,
                parameters,
            }))
            : args;
        const blockNumberHex = typeof blockNumber === 'bigint' ? (0, toHex_js_1.numberToHex)(blockNumber) : undefined;
        const block = blockNumberHex || blockTag;
        const rpcStateOverride = (0, stateOverride_js_1.serializeStateOverride)(stateOverride);
        const to = await (async () => {
            if (rest.to)
                return rest.to;
            if (authorizationList && authorizationList.length > 0)
                return await (0, recoverAuthorizationAddress_js_1.recoverAuthorizationAddress)({
                    authorization: authorizationList[0],
                }).catch(() => {
                    throw new base_js_1.BaseError('`to` is required. Could not infer from `authorizationList`');
                });
            return undefined;
        })();
        (0, assertRequest_js_1.assertRequest)(args);
        const chainFormat = client.chain?.formatters?.transactionRequest?.format;
        const format = chainFormat || transactionRequest_js_1.formatTransactionRequest;
        const request = format({
            ...(0, extract_js_1.extract)(rest, { format: chainFormat }),
            account,
            accessList,
            authorizationList,
            blobs,
            blobVersionedHashes,
            data,
            gas,
            gasPrice,
            maxFeePerBlobGas,
            maxFeePerGas,
            maxPriorityFeePerGas,
            nonce,
            to,
            value,
        }, 'estimateGas');
        return BigInt(await client.request({
            method: 'eth_estimateGas',
            params: rpcStateOverride
                ? [
                    request,
                    block ?? client.experimental_blockTag ?? 'latest',
                    rpcStateOverride,
                ]
                : block
                    ? [request, block]
                    : [request],
        }));
    }
    catch (err) {
        throw (0, getEstimateGasError_js_1.getEstimateGasError)(err, {
            ...args,
            account,
            chain: client.chain,
        });
    }
}
//# sourceMappingURL=estimateGas.js.map