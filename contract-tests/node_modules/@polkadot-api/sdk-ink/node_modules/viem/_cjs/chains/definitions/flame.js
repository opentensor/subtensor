"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.flame = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.flame = (0, defineChain_js_1.defineChain)({
    id: 253368190,
    name: 'Flame',
    network: 'flame',
    nativeCurrency: {
        symbol: 'TIA',
        name: 'TIA',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.flame.astria.org'],
            webSocket: ['wss://ws.flame.astria.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Flame Explorer',
            url: 'https://explorer.flame.astria.org',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 6829148,
        },
    },
});
//# sourceMappingURL=flame.js.map