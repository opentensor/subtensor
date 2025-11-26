"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.agungTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.agungTestnet = (0, defineChain_js_1.defineChain)({
    id: 9990,
    name: 'Agung Network',
    nativeCurrency: {
        decimals: 18,
        name: 'Agung',
        symbol: 'AGNG',
    },
    rpcUrls: {
        default: {
            http: ['https://wss-async.agung.peaq.network'],
            webSocket: ['wss://wss-async.agung.peaq.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Subscan',
            url: 'https://agung-testnet.subscan.io',
        },
    },
    testnet: true,
});
//# sourceMappingURL=agungTestnet.js.map