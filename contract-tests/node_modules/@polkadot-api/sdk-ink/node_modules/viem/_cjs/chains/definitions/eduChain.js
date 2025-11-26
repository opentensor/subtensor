"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.eduChain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.eduChain = (0, defineChain_js_1.defineChain)({
    id: 41923,
    name: 'EDU Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'EDU',
        symbol: 'EDU',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.edu-chain.raas.gelato.cloud'],
        },
    },
    blockExplorers: {
        default: {
            name: 'EDU Chain Explorer',
            url: 'https://educhain.blockscout.com/',
        },
    },
    testnet: false,
});
//# sourceMappingURL=eduChain.js.map