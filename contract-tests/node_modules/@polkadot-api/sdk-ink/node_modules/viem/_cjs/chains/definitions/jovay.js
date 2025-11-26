"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jovay = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.jovay = (0, defineChain_js_1.defineChain)({
    id: 5_734_951,
    name: 'Jovay Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://api.zan.top/public/jovay-mainnet'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Jovay Explorer',
            url: 'https://explorer.jovay.io',
        },
    },
    testnet: false,
});
//# sourceMappingURL=jovay.js.map