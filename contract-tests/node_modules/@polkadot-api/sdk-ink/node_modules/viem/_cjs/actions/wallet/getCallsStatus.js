"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getCallsStatus = getCallsStatus;
const slice_js_1 = require("../../utils/data/slice.js");
const trim_js_1 = require("../../utils/data/trim.js");
const fromHex_js_1 = require("../../utils/encoding/fromHex.js");
const transactionReceipt_js_1 = require("../../utils/formatters/transactionReceipt.js");
const sendCalls_js_1 = require("./sendCalls.js");
async function getCallsStatus(client, parameters) {
    async function getStatus(id) {
        const isTransactions = id.endsWith(sendCalls_js_1.fallbackMagicIdentifier.slice(2));
        if (isTransactions) {
            const chainId = (0, trim_js_1.trim)((0, slice_js_1.sliceHex)(id, -64, -32));
            const hashes = (0, slice_js_1.sliceHex)(id, 0, -64)
                .slice(2)
                .match(/.{1,64}/g);
            const receipts = await Promise.all(hashes.map((hash) => sendCalls_js_1.fallbackTransactionErrorMagicIdentifier.slice(2) !== hash
                ? client.request({
                    method: 'eth_getTransactionReceipt',
                    params: [`0x${hash}`],
                }, { dedupe: true })
                : undefined));
            const status = (() => {
                if (receipts.some((r) => r === null))
                    return 100;
                if (receipts.every((r) => r?.status === '0x1'))
                    return 200;
                if (receipts.every((r) => r?.status === '0x0'))
                    return 500;
                return 600;
            })();
            return {
                atomic: false,
                chainId: (0, fromHex_js_1.hexToNumber)(chainId),
                receipts: receipts.filter(Boolean),
                status,
                version: '2.0.0',
            };
        }
        return client.request({
            method: 'wallet_getCallsStatus',
            params: [id],
        });
    }
    const { atomic = false, chainId, receipts, version = '2.0.0', ...response } = await getStatus(parameters.id);
    const [status, statusCode] = (() => {
        const statusCode = response.status;
        if (statusCode >= 100 && statusCode < 200)
            return ['pending', statusCode];
        if (statusCode >= 200 && statusCode < 300)
            return ['success', statusCode];
        if (statusCode >= 300 && statusCode < 700)
            return ['failure', statusCode];
        if (statusCode === 'CONFIRMED')
            return ['success', 200];
        if (statusCode === 'PENDING')
            return ['pending', 100];
        return [undefined, statusCode];
    })();
    return {
        ...response,
        atomic,
        chainId: chainId ? (0, fromHex_js_1.hexToNumber)(chainId) : undefined,
        receipts: receipts?.map((receipt) => ({
            ...receipt,
            blockNumber: (0, fromHex_js_1.hexToBigInt)(receipt.blockNumber),
            gasUsed: (0, fromHex_js_1.hexToBigInt)(receipt.gasUsed),
            status: transactionReceipt_js_1.receiptStatuses[receipt.status],
        })) ?? [],
        statusCode,
        status,
        version,
    };
}
//# sourceMappingURL=getCallsStatus.js.map