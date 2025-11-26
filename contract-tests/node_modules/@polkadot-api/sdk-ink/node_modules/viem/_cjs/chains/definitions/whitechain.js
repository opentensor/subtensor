"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.whitechain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.whitechain = (0, defineChain_js_1.defineChain)({
    testnet: false,
    name: 'Whitechain',
    blockExplorers: {
        default: {
            name: 'Whitechain Explorer',
            url: 'https://explorer.whitechain.io',
        },
    },
    id: 1875,
    rpcUrls: {
        default: {
            http: ['https://rpc.whitechain.io'],
        },
    },
    nativeCurrency: {
        decimals: 18,
        name: 'WhiteBIT Coin',
        symbol: 'WBT',
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 25212237,
        },
    },
});
//# sourceMappingURL=whitechain.js.map