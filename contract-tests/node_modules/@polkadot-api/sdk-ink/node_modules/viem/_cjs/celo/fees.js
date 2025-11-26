"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fees = void 0;
exports.fees = {
    estimateFeesPerGas: async (params) => {
        if (!params.request?.feeCurrency)
            return null;
        const [gasPrice, maxPriorityFeePerGas] = await Promise.all([
            estimateFeePerGasInFeeCurrency(params.client, params.request.feeCurrency),
            estimateMaxPriorityFeePerGasInFeeCurrency(params.client, params.request.feeCurrency),
        ]);
        const maxFeePerGas = params.multiply(gasPrice - maxPriorityFeePerGas) + maxPriorityFeePerGas;
        return {
            maxFeePerGas,
            maxPriorityFeePerGas,
        };
    },
};
async function estimateFeePerGasInFeeCurrency(client, feeCurrency) {
    const fee = await client.request({
        method: 'eth_gasPrice',
        params: [feeCurrency],
    });
    return BigInt(fee);
}
async function estimateMaxPriorityFeePerGasInFeeCurrency(client, feeCurrency) {
    const feesPerGas = await client.request({
        method: 'eth_maxPriorityFeePerGas',
        params: [feeCurrency],
    });
    return BigInt(feesPerGas);
}
//# sourceMappingURL=fees.js.map