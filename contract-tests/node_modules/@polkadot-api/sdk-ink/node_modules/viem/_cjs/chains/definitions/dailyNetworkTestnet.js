"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.dailyNetworkTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.dailyNetworkTestnet = (0, defineChain_js_1.defineChain)({
    id: 825,
    name: 'Daily Network Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Daily',
        symbol: 'DLY',
    },
    rpcUrls: {
        default: { http: ['https://rpc.testnet.dailycrypto.net'] },
    },
    blockExplorers: {
        default: {
            name: 'Daily Testnet Explorer',
            url: 'https://explorer.testnet.dailycrypto.net',
        },
    },
    testnet: true,
});
//# sourceMappingURL=dailyNetworkTestnet.js.map