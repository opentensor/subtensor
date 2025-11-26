"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.edexaTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.edexaTestnet = (0, defineChain_js_1.defineChain)({
    id: 1995,
    name: 'edeXa Testnet',
    nativeCurrency: { name: 'edeXa', symbol: 'tEDX', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.edexa.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'edeXa Testnet Explorer',
            url: 'https://explorer.testnet.edexa.network',
            apiUrl: 'https://explorer.testnet.edexa.network/api/v2',
        },
    },
    testnet: true,
});
//# sourceMappingURL=edexaTestnet.js.map