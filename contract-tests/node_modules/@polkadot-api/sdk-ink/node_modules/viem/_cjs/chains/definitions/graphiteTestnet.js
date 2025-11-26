"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.graphiteTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.graphiteTestnet = (0, defineChain_js_1.defineChain)({
    id: 54170,
    name: 'Graphite Network Testnet',
    nativeCurrency: { name: 'Graphite', symbol: '@G', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://anon-entrypoint-test-1.atgraphite.com'],
            webSocket: ['wss://ws-anon-entrypoint-test-1.atgraphite.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Graphite Testnet Spectre',
            url: 'https://test.atgraphite.com',
            apiUrl: 'https://api.test.atgraphite.com/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=graphiteTestnet.js.map