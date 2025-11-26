"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nomina = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.nomina = (0, defineChain_js_1.defineChain)({
    id: 166,
    name: 'Nomina',
    nativeCurrency: {
        name: 'Nomina',
        symbol: 'NOM',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://mainnet.nomina.io'],
            webSocket: ['wss://mainnet.nomina.io'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Nomina Explorer',
            url: 'https://nomscan.io',
        },
    },
    testnet: false,
});
//# sourceMappingURL=nomina.js.map