"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.exsatTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.exsatTestnet = (0, defineChain_js_1.defineChain)({
    id: 839999,
    name: 'exSat Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'BTC',
        symbol: 'BTC',
    },
    rpcUrls: {
        default: { http: ['https://evm-tst3.exsat.network'] },
    },
    blockExplorers: {
        default: {
            name: 'exSat Explorer',
            url: 'https://scan-testnet.exsat.network',
            apiUrl: 'https://scan-testnet.exsat.network/api',
        },
    },
});
//# sourceMappingURL=exSatTestnet.js.map