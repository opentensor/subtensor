"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.birdlayer = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.birdlayer = (0, defineChain_js_1.defineChain)({
    id: 53456,
    name: 'BirdLayer',
    nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
    rpcUrls: {
        default: {
            http: ['https://rpc.birdlayer.xyz', 'https://rpc1.birdlayer.xyz'],
            webSocket: ['wss://rpc.birdlayer.xyz/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'BirdLayer Explorer',
            url: 'https://scan.birdlayer.xyz',
        },
    },
});
//# sourceMappingURL=birdlayer.js.map