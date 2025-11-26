"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoSGD1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoSGD1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_012,
    name: 'Juneo SGD1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo SGD1-Chain',
        symbol: 'SGD1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/SGD1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/7',
            apiUrl: 'https://juneoscan.io/chain/7/api',
        },
    },
});
//# sourceMappingURL=juneoSGD1Chain.js.map