"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.plumeDevnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 11_155_111;
exports.plumeDevnet = (0, defineChain_js_1.defineChain)({
    id: 98_864,
    name: 'Plume Devnet (Legacy)',
    nativeCurrency: {
        name: 'Plume Sepolia Ether',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://test-rpc.plumenetwork.xyz'],
            webSocket: ['wss://test-rpc.plumenetwork.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://test-explorer.plumenetwork.xyz',
            apiUrl: 'https://test-explorer.plumenetwork.xyz/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 481_948,
        },
    },
    testnet: true,
    sourceId,
});
//# sourceMappingURL=plumeDevnet.js.map