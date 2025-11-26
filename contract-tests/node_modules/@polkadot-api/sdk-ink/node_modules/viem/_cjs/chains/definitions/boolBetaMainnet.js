"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.boolBetaMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.boolBetaMainnet = (0, defineChain_js_1.defineChain)({
    id: 11100,
    name: 'Bool Beta Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'BOL',
        symbol: 'BOL',
    },
    rpcUrls: {
        default: { http: ['https://beta-rpc-node-http.bool.network'] },
    },
    blockExplorers: {
        default: {
            name: 'BoolScan',
            url: 'https://beta-mainnet.boolscan.com/',
        },
    },
    testnet: false,
});
//# sourceMappingURL=boolBetaMainnet.js.map