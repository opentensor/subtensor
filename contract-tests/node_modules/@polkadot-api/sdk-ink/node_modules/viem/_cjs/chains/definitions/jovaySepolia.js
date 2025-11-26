"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jovaySepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.jovaySepolia = (0, defineChain_js_1.defineChain)({
    id: 2_019_775,
    name: 'Jovay Sepolia Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://api.zan.top/public/jovay-testnet'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Jovay Testnet Explorer',
            url: 'https://sepolia-explorer.jovay.io/l2',
        },
    },
    testnet: true,
});
//# sourceMappingURL=jovaySepolia.js.map