"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.forta = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.forta = (0, defineChain_js_1.defineChain)({
    id: 80_931,
    name: 'Forta Chain',
    nativeCurrency: {
        symbol: 'FORT',
        name: 'FORT',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc-forta-chain-8gj1qndmfc.t.conduit.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Forta Explorer',
            url: 'https://explorer.forta.org',
        },
    },
});
//# sourceMappingURL=forta.js.map