"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoEUR1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoEUR1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_011,
    name: 'Juneo EUR1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo EUR1-Chain',
        symbol: 'EUR1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/EUR1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/6',
            apiUrl: 'https://juneoscan.io/chain/6/api',
        },
    },
});
//# sourceMappingURL=juneoEUR1Chain.js.map