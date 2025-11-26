"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoUSDT1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoUSDT1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_005,
    name: 'Juneo USDT1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo USDT1-Chain',
        symbol: 'USDT1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/USDT1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/3',
            apiUrl: 'https://juneoscan.io/chain/3/api',
        },
    },
});
//# sourceMappingURL=juneoUSDT1Chain.js.map