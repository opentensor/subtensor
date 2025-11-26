"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fireChain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.fireChain = (0, defineChain_js_1.defineChain)({
    id: 995,
    name: '5ireChain',
    nativeCurrency: { name: '5ire Token', symbol: '5IRE', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.5ire.network'],
        },
    },
    blockExplorers: {
        default: {
            name: '5ireChain Mainnet Explorer',
            url: 'https://5irescan.io/',
        },
    },
    testnet: false,
});
//# sourceMappingURL=5ireChain.js.map