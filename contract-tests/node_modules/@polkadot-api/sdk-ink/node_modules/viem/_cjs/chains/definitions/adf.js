"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.adf = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.adf = (0, defineChain_js_1.defineChain)({
    id: 1215,
    name: 'ADF Chain',
    nativeCurrency: { name: 'ADDFILL', symbol: 'ADF', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet.adftechnology.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'ADF Mainnet Explorer',
            url: 'https://explorer.adftechnology.com',
        },
    },
    testnet: false,
});
//# sourceMappingURL=adf.js.map