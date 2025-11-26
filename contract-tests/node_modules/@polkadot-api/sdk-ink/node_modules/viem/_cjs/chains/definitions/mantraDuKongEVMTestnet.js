"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mantraDuKongEVMTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.mantraDuKongEVMTestnet = (0, defineChain_js_1.defineChain)({
    id: 5887,
    name: 'MANTRA DuKong EVM Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'OM',
        symbol: 'OM',
    },
    rpcUrls: {
        default: { http: ['https://evm.dukong.mantrachain.io'] },
    },
    blockExplorers: {
        default: {
            name: 'MANTRAScan',
            url: 'https://mantrascan.io/dukong',
        },
    },
    testnet: true,
});
//# sourceMappingURL=mantraDuKongEVMTestnet.js.map