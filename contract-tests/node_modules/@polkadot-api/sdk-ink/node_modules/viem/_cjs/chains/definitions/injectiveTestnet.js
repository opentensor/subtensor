"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.injectiveTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.injectiveTestnet = (0, defineChain_js_1.defineChain)({
    id: 1439,
    name: 'Injective Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Injective',
        symbol: 'INJ',
    },
    rpcUrls: {
        default: {
            http: ['https://k8s.testnet.json-rpc.injective.network'],
            webSocket: ['wss://k8s.testnet.ws.injective.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Injective Explorer',
            url: 'https://testnet.blockscout.injective.network',
            apiUrl: 'https://testnet.blockscout.injective.network/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=injectiveTestnet.js.map