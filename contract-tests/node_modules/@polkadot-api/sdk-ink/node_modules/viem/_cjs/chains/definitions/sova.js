"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sova = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sova = (0, defineChain_js_1.defineChain)({
    id: 100_021,
    name: 'Sova',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.sova.io'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Sova Block Explorer',
            url: 'hhttps://explorer.sova.io',
        },
    },
    testnet: false,
});
//# sourceMappingURL=sova.js.map