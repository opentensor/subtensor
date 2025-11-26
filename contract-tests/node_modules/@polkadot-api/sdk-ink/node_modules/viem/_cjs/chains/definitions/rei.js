"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rei = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.rei = (0, defineChain_js_1.defineChain)({
    id: 47805,
    name: 'REI Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'REI',
        symbol: 'REI',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.rei.network'],
            webSocket: ['wss://rpc.rei.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'REI Scan',
            url: 'https://scan.rei.network',
        },
    },
    testnet: false,
});
//# sourceMappingURL=rei.js.map