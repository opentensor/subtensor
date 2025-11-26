"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.plume = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.plume = (0, defineChain_js_1.defineChain)({
    id: 98_865,
    name: 'Plume Mainnet',
    nativeCurrency: {
        name: 'Plume Ether',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.plumenetwork.xyz'],
            webSocket: ['wss://rpc.plumenetwork.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://explorer.plumenetwork.xyz',
            apiUrl: 'https://explorer.plumenetwork.xyz/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 48_577,
        },
    },
    sourceId,
});
//# sourceMappingURL=plume.js.map