"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nearTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.nearTestnet = (0, defineChain_js_1.defineChain)({
    id: 398,
    name: 'NEAR Protocol Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'NEAR',
        symbol: 'NEAR',
    },
    rpcUrls: {
        default: { http: ['https://eth-rpc.testnet.near.org'] },
    },
    blockExplorers: {
        default: {
            name: 'NEAR Explorer',
            url: 'https://eth-explorer-testnet.near.org',
        },
    },
    testnet: true,
});
//# sourceMappingURL=nearTestnet.js.map