"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.huddle01Testnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 421_614;
exports.huddle01Testnet = (0, defineChain_js_1.defineChain)({
    id: 2524852,
    name: 'Huddle01 dRTC Chain Testnet',
    nativeCurrency: {
        name: 'Ethereum',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://huddle-testnet.rpc.caldera.xyz/http'],
            webSocket: ['wss://huddle-testnet.rpc.caldera.xyz/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Huddle01 Caldera Explorer',
            url: 'https://huddle-testnet.explorer.caldera.xyz',
            apiUrl: 'https://huddle-testnet.explorer.caldera.xyz/api',
        },
    },
    sourceId,
});
//# sourceMappingURL=huddle01Testnet.js.map