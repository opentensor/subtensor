"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zksyncLocalHyperchain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const chainConfig_js_1 = require("../../zksync/chainConfig.js");
exports.zksyncLocalHyperchain = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 270,
    name: 'ZKsync CLI Local Hyperchain',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['http://localhost:15100'],
            webSocket: ['ws://localhost:15101'],
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
//# sourceMappingURL=zksyncLocalHyperchain.js.map