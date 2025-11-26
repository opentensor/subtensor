"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.metisSepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.metisSepolia = (0, defineChain_js_1.defineChain)({
    id: 59902,
    name: 'Metis Sepolia',
    nativeCurrency: {
        decimals: 18,
        name: 'Test Metis',
        symbol: 'tMETIS',
    },
    rpcUrls: {
        default: {
            http: [
                'wss://metis-sepolia-rpc.publicnode.com',
                'https://sepolia.metisdevops.link',
                'https://metis-sepolia-rpc.publicnode.com',
                'https://metis-sepolia.gateway.tenderly.co',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Metis Sepolia Explorer',
            url: 'https://sepolia-explorer.metisdevops.link',
            apiUrl: 'https://sepolia-explorer.metisdevops.link/api-docs',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 224185,
        },
    },
});
//# sourceMappingURL=metisSepolia.js.map