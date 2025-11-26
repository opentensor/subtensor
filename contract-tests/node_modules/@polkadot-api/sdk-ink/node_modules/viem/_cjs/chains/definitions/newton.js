"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.newton = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.newton = (0, defineChain_js_1.defineChain)({
    id: 1012,
    name: 'Newton',
    nativeCurrency: {
        name: 'Newton',
        symbol: 'NEW',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://global.rpc.mainnet.newtonproject.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'NewFi explorer',
            url: 'https://explorer.newtonproject.org/',
        },
    },
    testnet: false,
});
//# sourceMappingURL=newton.js.map