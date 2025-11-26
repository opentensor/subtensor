"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.reddioSepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.reddioSepolia = (0, defineChain_js_1.defineChain)({
    id: 50341,
    name: 'Reddio Sepolia',
    nativeCurrency: { name: 'Reddio', symbol: 'RED', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://reddio-dev.reddio.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Reddioscan',
            url: 'https://reddio-devnet.l2scan.co',
            apiUrl: 'https://reddio-devnet.l2scan.co/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=reddioSepolia.js.map