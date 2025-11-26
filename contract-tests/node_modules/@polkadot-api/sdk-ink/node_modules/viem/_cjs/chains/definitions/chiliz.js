"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.chiliz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.chiliz = (0, defineChain_js_1.defineChain)({
    id: 88_888,
    name: 'Chiliz Chain',
    network: 'chiliz-chain',
    nativeCurrency: {
        decimals: 18,
        name: 'CHZ',
        symbol: 'CHZ',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.chiliz.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Chiliz Explorer',
            url: 'https://scan.chiliz.com',
            apiUrl: 'https://scan.chiliz.com/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 8080847,
        },
    },
});
//# sourceMappingURL=chiliz.js.map