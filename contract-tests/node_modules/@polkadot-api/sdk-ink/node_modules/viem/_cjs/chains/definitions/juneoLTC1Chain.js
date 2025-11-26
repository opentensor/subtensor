"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoLTC1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoLTC1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_009,
    name: 'Juneo LTC1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo LTC1-Chain',
        symbol: 'LTC1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/LTC1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/11',
            apiUrl: 'https://juneoscan.io/chain/11/api',
        },
    },
});
//# sourceMappingURL=juneoLTC1Chain.js.map