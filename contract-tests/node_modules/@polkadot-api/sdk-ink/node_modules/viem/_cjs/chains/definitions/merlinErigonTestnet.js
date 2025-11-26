"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.merlinErigonTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.merlinErigonTestnet = (0, defineChain_js_1.defineChain)({
    id: 4203,
    name: 'Merlin Erigon Testnet',
    nativeCurrency: {
        name: 'BTC',
        symbol: 'BTC',
        decimals: 18,
    },
    rpcUrls: {
        default: { http: ['https://testnet-erigon-rpc.merlinchain.io'] },
    },
    blockExplorers: {
        default: {
            name: 'blockscout',
            url: 'https://testnet-erigon-scan.merlinchain.io',
            apiUrl: 'https://testnet-erigon-scan.merlinchain.io/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=merlinErigonTestnet.js.map