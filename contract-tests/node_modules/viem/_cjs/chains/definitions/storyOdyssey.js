"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.storyOdyssey = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.storyOdyssey = (0, defineChain_js_1.defineChain)({
    id: 1516,
    name: 'Story Odyssey',
    nativeCurrency: {
        decimals: 18,
        name: 'IP',
        symbol: 'IP',
    },
    rpcUrls: {
        default: { http: ['https://rpc.odyssey.storyrpc.io'] },
    },
    blockExplorers: {
        default: {
            name: 'Story Odyssey Explorer',
            url: 'https://odyssey.storyscan.xyz',
        },
    },
    testnet: true,
});
//# sourceMappingURL=storyOdyssey.js.map