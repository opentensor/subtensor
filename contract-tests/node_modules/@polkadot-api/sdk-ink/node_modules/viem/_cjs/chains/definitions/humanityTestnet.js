"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.humanityTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.humanityTestnet = (0, defineChain_js_1.defineChain)({
    id: 7_080_969,
    name: 'Humanity Testnet',
    nativeCurrency: {
        name: 'tHP',
        symbol: 'tHP',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.humanity.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Humanity Testnet Explorer',
            url: 'https://humanity-testnet.explorer.alchemy.com',
            apiUrl: 'https://humanity-testnet.explorer.alchemy.com/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=humanityTestnet.js.map