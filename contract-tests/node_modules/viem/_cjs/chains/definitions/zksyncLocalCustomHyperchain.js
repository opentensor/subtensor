"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zksyncLocalCustomHyperchain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const chainConfig_js_1 = require("../../zksync/chainConfig.js");
exports.zksyncLocalCustomHyperchain = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 272,
    name: 'ZKsync CLI Local Custom Hyperchain',
    nativeCurrency: { name: 'BAT', symbol: 'BAT', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['http://localhost:15200'],
            webSocket: ['ws://localhost:15201'],
        },
    },
    blockExplorers: {
        default: {
            name: 'ZKsync explorer',
            url: 'http://localhost:15005/',
            apiUrl: 'http://localhost:15005/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=zksyncLocalCustomHyperchain.js.map