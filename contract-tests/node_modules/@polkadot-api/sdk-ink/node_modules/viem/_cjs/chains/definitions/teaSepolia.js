"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.teaSepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.teaSepolia = (0, defineChain_js_1.defineChain)({
    id: 10_218,
    name: 'Tea Sepolia',
    nativeCurrency: { name: 'Sepolia Tea', symbol: 'TEA', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://tea-sepolia.g.alchemy.com/public'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Tea Sepolia Explorer',
            url: 'https://sepolia.tea.xyz',
        },
    },
    testnet: true,
});
//# sourceMappingURL=teaSepolia.js.map