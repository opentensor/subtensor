"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.dailyNetwork = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.dailyNetwork = (0, defineChain_js_1.defineChain)({
    id: 824,
    name: 'Daily Network Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Daily',
        symbol: 'DLY',
    },
    rpcUrls: {
        default: { http: ['https://rpc.mainnet.dailycrypto.net'] },
    },
    blockExplorers: {
        default: {
            name: 'Daily Mainnet Explorer',
            url: 'https://explorer.mainnet.dailycrypto.net',
        },
    },
    testnet: false,
});
//# sourceMappingURL=dailyNetwork.js.map