"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.tronNile = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.tronNile = (0, defineChain_js_1.defineChain)({
    id: 3448148188,
    name: 'Tron Nile',
    nativeCurrency: { name: 'TRON', symbol: 'TRX', decimals: 6 },
    rpcUrls: {
        default: {
            http: ['https://nile.trongrid.io/jsonrpc'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Tronscan',
            url: 'https://nile.tronscan.org',
        },
    },
    testnet: true,
});
//# sourceMappingURL=tronNile.js.map