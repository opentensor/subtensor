"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sendRawTransactionSync = sendRawTransactionSync;
const transaction_js_1 = require("../../errors/transaction.js");
const transactionReceipt_js_1 = require("../../utils/formatters/transactionReceipt.js");
const index_js_1 = require("../../utils/index.js");
async function sendRawTransactionSync(client, { serializedTransaction, throwOnReceiptRevert, timeout, }) {
    const receipt = await client.request({
        method: 'eth_sendRawTransactionSync',
        params: timeout
            ? [serializedTransaction, (0, index_js_1.numberToHex)(timeout)]
            : [serializedTransaction],
    }, { retryCount: 0 });
    const format = client.chain?.formatters?.transactionReceipt?.format ||
        transactionReceipt_js_1.formatTransactionReceipt;
    const formatted = format(receipt);
    if (formatted.status === 'reverted' && throwOnReceiptRevert)
        throw new transaction_js_1.TransactionReceiptRevertedError({ receipt: formatted });
    return formatted;
}
//# sourceMappingURL=sendRawTransactionSync.js.map