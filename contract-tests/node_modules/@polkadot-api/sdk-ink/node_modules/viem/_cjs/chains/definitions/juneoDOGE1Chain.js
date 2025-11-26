"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoDOGE1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoDOGE1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_010,
    name: 'Juneo DOGE1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo DOGE1-Chain',
        symbol: 'DOGE1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/DOGE1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/10',
            apiUrl: 'https://juneoscan.io/chain/10/api',
        },
    },
});
//# sourceMappingURL=juneoDOGE1Chain.js.map