"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.whitechainTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.whitechainTestnet = (0, defineChain_js_1.defineChain)({
    testnet: true,
    name: 'Whitechain Testnet',
    blockExplorers: {
        default: {
            name: 'Whitechain Explorer',
            url: 'https://testnet.whitechain.io',
        },
    },
    id: 2625,
    rpcUrls: {
        default: {
            http: ['https://rpc-testnet.whitechain.io'],
        },
    },
    nativeCurrency: {
        decimals: 18,
        name: 'WhiteBIT Coin',
        symbol: 'WBT',
    },
});
//# sourceMappingURL=whitechainTestnet.js.map