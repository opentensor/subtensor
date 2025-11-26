"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ultra = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.ultra = (0, defineChain_js_1.defineChain)({
    id: 19991,
    name: 'Ultra EVM',
    nativeCurrency: {
        decimals: 18,
        name: 'Ultra Token',
        symbol: 'UOS',
    },
    rpcUrls: {
        default: { http: ['https://evm.ultra.eosusa.io'] },
    },
    blockExplorers: {
        default: {
            name: 'Ultra EVM Explorer',
            url: 'https://evmexplorer.ultra.io',
        },
    },
});
//# sourceMappingURL=ultra.js.map