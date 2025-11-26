"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoUSD1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoUSD1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_006,
    name: 'Juneo USD1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo USD1-Chain',
        symbol: 'USD1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/USD1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/4',
            apiUrl: 'https://juneoscan.io/chain/4/api',
        },
    },
});
//# sourceMappingURL=juneoUSD1Chain.js.map