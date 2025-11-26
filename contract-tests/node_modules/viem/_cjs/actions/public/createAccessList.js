"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createAccessList = createAccessList;
const parseAccount_js_1 = require("../../accounts/utils/parseAccount.js");
const toHex_js_1 = require("../../utils/encoding/toHex.js");
const getCallError_js_1 = require("../../utils/errors/getCallError.js");
const extract_js_1 = require("../../utils/formatters/extract.js");
const transactionRequest_js_1 = require("../../utils/formatters/transactionRequest.js");
const assertRequest_js_1 = require("../../utils/transaction/assertRequest.js");
async function createAccessList(client, args) {
    const { account: account_ = client.account, blockNumber, blockTag = 'latest', blobs, data, gas, gasPrice, maxFeePerBlobGas, maxFeePerGas, maxPriorityFeePerGas, to, value, ...rest } = args;
    const account = account_ ? (0, parseAccount_js_1.parseAccount)(account_) : undefined;
    try {
        (0, assertRequest_js_1.assertRequest)(args);
        const blockNumberHex = blockNumber ? (0, toHex_js_1.numberToHex)(blockNumber) : undefined;
        const block = blockNumberHex || blockTag;
        const chainFormat = client.chain?.formatters?.transactionRequest?.format;
        const format = chainFormat || transactionRequest_js_1.formatTransactionRequest;
        const request = format({
            ...(0, extract_js_1.extract)(rest, { format: chainFormat }),
            from: account?.address,
            blobs,
            data,
            gas,
            gasPrice,
            maxFeePerBlobGas,
            maxFeePerGas,
            maxPriorityFeePerGas,
            to,
            value,
        });
        const response = await client.request({
            method: 'eth_createAccessList',
            params: [request, block],
        });
        return {
            accessList: response.accessList,
            gasUsed: BigInt(response.gasUsed),
        };
    }
    catch (err) {
        throw (0, getCallError_js_1.getCallError)(err, {
            ...args,
            account,
            chain: client.chain,
        });
    }
}
//# sourceMappingURL=createAccessList.js.map