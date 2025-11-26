"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ultraTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.ultraTestnet = (0, defineChain_js_1.defineChain)({
    id: 18881,
    name: 'Ultra EVM Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Ultra Token',
        symbol: 'UOS',
    },
    rpcUrls: {
        default: { http: ['https://evm.test.ultra.eosusa.io'] },
    },
    blockExplorers: {
        default: {
            name: 'Ultra EVM Testnet Explorer',
            url: 'https://evmexplorer.testnet.ultra.io',
        },
    },
    testnet: true,
});
//# sourceMappingURL=ultraTestnet.js.map