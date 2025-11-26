"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.haustTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.haustTestnet = (0, defineChain_js_1.defineChain)({
    id: 1_523_903_251,
    name: 'Haust Network Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'HAUST',
        symbol: 'HAUST',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc-testnet.haust.app'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Haust Network Testnet Explorer',
            url: 'https://explorer-testnet.haust.app',
        },
    },
    testnet: true,
});
//# sourceMappingURL=haustTestnet.js.map