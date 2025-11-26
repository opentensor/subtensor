"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ethernity = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.ethernity = (0, defineChain_js_1.defineChain)({
    id: 183,
    name: 'Ethernity',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: { http: ['https://mainnet.ethernitychain.io'] },
    },
    blockExplorers: {
        default: {
            name: 'Ethernity Explorer',
            url: 'https://ernscan.io',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 0,
        },
    },
    testnet: false,
});
//# sourceMappingURL=ethernity.js.map