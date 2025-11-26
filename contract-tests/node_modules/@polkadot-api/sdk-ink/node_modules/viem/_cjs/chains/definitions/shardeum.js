"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.shardeum = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.shardeum = (0, defineChain_js_1.defineChain)({
    id: 8118,
    name: 'Shardeum',
    nativeCurrency: { name: 'Shardeum', symbol: 'SHM', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://api.shardeum.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Shardeum Explorer',
            url: 'https://explorer.shardeum.org',
        },
    },
    testnet: false,
});
//# sourceMappingURL=shardeum.js.map