"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.exsat = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.exsat = (0, defineChain_js_1.defineChain)({
    id: 7200,
    name: 'exSat Network',
    nativeCurrency: {
        decimals: 18,
        name: 'BTC',
        symbol: 'BTC',
    },
    rpcUrls: {
        default: { http: ['https://evm.exsat.network'] },
    },
    blockExplorers: {
        default: {
            name: 'exSat Explorer',
            url: 'https://scan.exsat.network',
            apiUrl: 'https://scan.exsat.network/api',
        },
    },
});
//# sourceMappingURL=exSat.js.map