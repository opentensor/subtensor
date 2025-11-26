"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.botanix = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.botanix = (0, defineChain_js_1.defineChain)({
    id: 3637,
    name: 'Botanix',
    nativeCurrency: { name: 'Bitcoin', symbol: 'BTC', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.botanixlabs.com'],
            webSocket: ['wss://rpc.botanixlabs.com/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Botanixscan',
            url: 'https://botanixscan.io',
        },
    },
});
//# sourceMappingURL=botanix.js.map