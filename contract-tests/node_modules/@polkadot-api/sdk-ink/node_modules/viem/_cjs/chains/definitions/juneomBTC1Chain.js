"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneomBTC1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneomBTC1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_007,
    name: 'Juneo mBTC1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo mBTC1-Chain',
        symbol: 'mBTC1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/mBTC1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/9',
            apiUrl: 'https://juneoscan.io/chain/9/api',
        },
    },
});
//# sourceMappingURL=juneomBTC1Chain.js.map