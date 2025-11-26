"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lestnet = (0, defineChain_js_1.defineChain)({
    id: 21363,
    name: 'Lestnet',
    nativeCurrency: { name: 'Lestnet Ether', symbol: 'LETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://service.lestnet.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Lestnet Explorer',
            url: 'https://explore.lestnet.org',
        },
    },
    testnet: true,
});
//# sourceMappingURL=lestnet.js.map