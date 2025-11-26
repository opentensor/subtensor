"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hemi = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.hemi = (0, defineChain_js_1.defineChain)({
    id: 43111,
    name: 'Hemi',
    network: 'Hemi',
    blockTime: 12_000,
    nativeCurrency: {
        name: 'Ether',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.hemi.network/rpc'],
        },
    },
    blockExplorers: {
        default: {
            name: 'blockscout',
            url: 'https://explorer.hemi.xyz',
        },
    },
    testnet: false,
});
//# sourceMappingURL=hemi.js.map