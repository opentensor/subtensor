"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.plumeSepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 11_155_111;
exports.plumeSepolia = (0, defineChain_js_1.defineChain)({
    id: 98_867,
    name: 'Plume Testnet',
    nativeCurrency: {
        name: 'Plume',
        symbol: 'PLUME',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://testnet-rpc.plume.org'],
            webSocket: ['wss://testnet-rpc.plume.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://testnet-explorer.plume.org',
            apiUrl: 'https://testnet-explorer.plume.org/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 199_712,
        },
    },
    testnet: true,
    sourceId,
});
//# sourceMappingURL=plumeSepolia.js.map