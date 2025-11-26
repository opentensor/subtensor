"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.citreaTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.citreaTestnet = (0, defineChain_js_1.defineChain)({
    id: 5115,
    name: 'Citrea Testnet',
    nativeCurrency: { name: 'cBTC', symbol: 'cBTC', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.citrea.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Citrea Explorer',
            url: 'https://explorer.testnet.citrea.xyz',
            apiUrl: 'https://explorer.testnet.citrea.xyz/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=citreaTestnet.js.map