"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.injective = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.injective = (0, defineChain_js_1.defineChain)({
    id: 1776,
    name: 'Injective',
    nativeCurrency: {
        decimals: 18,
        name: 'Injective',
        symbol: 'INJ',
    },
    rpcUrls: {
        default: {
            http: ['https://sentry.evm-rpc.injective.network'],
            webSocket: ['wss://sentry.evm-ws.injective.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Injective Explorer',
            url: 'https://blockscout.injective.network',
            apiUrl: 'https://blockscout.injective.network/api',
        },
    },
    testnet: false,
});
//# sourceMappingURL=injective.js.map