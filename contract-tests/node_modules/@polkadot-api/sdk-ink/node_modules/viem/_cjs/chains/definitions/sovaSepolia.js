"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sovaSepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sovaSepolia = (0, defineChain_js_1.defineChain)({
    id: 120_893,
    name: 'Sova Network Sepolia',
    nativeCurrency: {
        decimals: 18,
        name: 'Sepolia Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.sova.io'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Sova Sepolia Explorer',
            url: 'https://explorer.testnet.sova.io',
        },
    },
    testnet: true,
});
//# sourceMappingURL=sovaSepolia.js.map