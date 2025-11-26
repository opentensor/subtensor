"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.humanity = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.humanity = (0, defineChain_js_1.defineChain)({
    id: 6_985_385,
    name: 'Humanity',
    nativeCurrency: {
        name: 'H',
        symbol: 'H',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://humanity-mainnet.g.alchemy.com/public'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Humanity Mainnet Explorer',
            url: 'https://humanity-mainnet.explorer.alchemy.com',
            apiUrl: 'https://humanity-mainnet.explorer.alchemy.com/api',
        },
    },
    testnet: false,
});
//# sourceMappingURL=humanity.js.map