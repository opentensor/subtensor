"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.loadAlphanet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.loadAlphanet = (0, defineChain_js_1.defineChain)({
    id: 9496,
    name: 'Load Alphanet',
    nativeCurrency: { name: 'Testnet LOAD', symbol: 'tLOAD', decimals: 18 },
    rpcUrls: {
        default: { http: ['https://alphanet.load.network'] },
    },
    blockExplorers: {
        default: {
            name: 'Load Alphanet Explorer',
            url: 'https://explorer.load.network',
        },
    },
    testnet: true,
});
//# sourceMappingURL=loadAlphanet.js.map