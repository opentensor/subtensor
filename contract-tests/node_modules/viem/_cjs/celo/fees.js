"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fees = void 0;
const getCode_js_1 = require("../actions/public/getCode.js");
exports.fees = {
    estimateFeesPerGas: async (params) => {
        if (!params.request?.feeCurrency)
            return null;
        const [gasPrice, maxPriorityFeePerGas, cel2] = await Promise.all([
            estimateFeePerGasInFeeCurrency(params.client, params.request.feeCurrency),
            estimateMaxPriorityFeePerGasInFeeCurrency(params.client, params.request.feeCurrency),
            isCel2(params.client),
        ]);
        const maxFeePerGas = cel2
            ?
                params.multiply(gasPrice - maxPriorityFeePerGas) + maxPriorityFeePerGas
            :
                gasPrice + maxPriorityFeePerGas;
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
async function isCel2(client) {
    const proxyAdminAddress = '0x4200000000000000000000000000000000000018';
    const code = await (0, getCode_js_1.getCode)(client, { address: proxyAdminAddress });
    return Boolean(code);
}
//# sourceMappingURL=fees.js.map