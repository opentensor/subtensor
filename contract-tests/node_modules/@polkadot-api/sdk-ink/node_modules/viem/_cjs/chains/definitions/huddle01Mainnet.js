"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.huddle01Mainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 42_161;
exports.huddle01Mainnet = (0, defineChain_js_1.defineChain)({
    id: 12323,
    name: 'Huddle01 dRTC Chain',
    nativeCurrency: {
        name: 'Ethereum',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://huddle01.calderachain.xyz/http'],
            webSocket: ['wss://huddle01.calderachain.xyz/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Huddle01 Caldera Explorer',
            url: 'https://huddle01.calderaexplorer.xyz',
            apiUrl: 'https://huddle01.calderaexplorer.xyz/api',
        },
    },
    sourceId,
});
//# sourceMappingURL=huddle01Mainnet.js.map