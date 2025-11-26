"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sonic = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sonic = (0, defineChain_js_1.defineChain)({
    id: 146,
    name: 'Sonic',
    blockTime: 630,
    nativeCurrency: {
        decimals: 18,
        name: 'Sonic',
        symbol: 'S',
    },
    rpcUrls: {
        default: { http: ['https://rpc.soniclabs.com'] },
    },
    blockExplorers: {
        default: {
            name: 'Sonic Explorer',
            url: 'https://sonicscan.org',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 60,
        },
    },
    testnet: false,
});
//# sourceMappingURL=sonic.js.map