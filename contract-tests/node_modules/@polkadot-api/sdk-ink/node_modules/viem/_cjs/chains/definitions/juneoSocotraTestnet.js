"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoSocotraTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoSocotraTestnet = (0, defineChain_js_1.defineChain)({
    id: 101_003,
    name: 'Socotra JUNE-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Socotra JUNE-Chain',
        symbol: 'JUNE',
    },
    rpcUrls: {
        default: { http: ['https://rpc.socotra-testnet.network/ext/bc/JUNE/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://socotra.juneoscan.io/chain/2',
            apiUrl: 'https://socotra.juneoscan.io/chain/2/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=juneoSocotraTestnet.js.map