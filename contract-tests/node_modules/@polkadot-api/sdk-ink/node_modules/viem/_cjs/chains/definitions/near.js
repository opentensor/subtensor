"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.near = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.near = (0, defineChain_js_1.defineChain)({
    id: 397,
    name: 'NEAR Protocol',
    nativeCurrency: {
        decimals: 18,
        name: 'NEAR',
        symbol: 'NEAR',
    },
    rpcUrls: {
        default: { http: ['https://eth-rpc.mainnet.near.org'] },
    },
    blockExplorers: {
        default: {
            name: 'NEAR Explorer',
            url: 'https://eth-explorer.near.org',
        },
    },
    testnet: false,
});
//# sourceMappingURL=near.js.map