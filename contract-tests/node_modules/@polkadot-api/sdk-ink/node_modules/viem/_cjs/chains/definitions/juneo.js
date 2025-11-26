"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneo = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneo = (0, defineChain_js_1.defineChain)({
    id: 45_003,
    name: 'Juneo JUNE-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'JUNE-Chain',
        symbol: 'JUNE',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/JUNE/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/2',
            apiUrl: 'https://juneoscan.io/chain/2/api',
        },
    },
});
//# sourceMappingURL=juneo.js.map