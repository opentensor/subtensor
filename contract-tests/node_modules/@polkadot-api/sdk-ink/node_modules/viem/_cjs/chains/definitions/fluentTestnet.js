"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fluentTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.fluentTestnet = (0, defineChain_js_1.defineChain)({
    id: 20_993,
    name: 'Fluent Testnet',
    nativeCurrency: {
        name: 'Ether',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.dev.gblend.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Fluent Explorer',
            url: 'https://blockscout.dev.gblend.xyz',
        },
    },
    testnet: true,
});
//# sourceMappingURL=fluentTestnet.js.map