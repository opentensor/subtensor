"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.graphite = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.graphite = (0, defineChain_js_1.defineChain)({
    id: 440017,
    name: 'Graphite Network',
    nativeCurrency: { name: 'Graphite', symbol: '@G', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://anon-entrypoint-1.atgraphite.com'],
            webSocket: ['wss://ws-anon-entrypoint-1.atgraphite.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Graphite Spectre',
            url: 'https://main.atgraphite.com',
            apiUrl: 'https://api.main.atgraphite.com/api',
        },
    },
    testnet: false,
});
//# sourceMappingURL=graphite.js.map