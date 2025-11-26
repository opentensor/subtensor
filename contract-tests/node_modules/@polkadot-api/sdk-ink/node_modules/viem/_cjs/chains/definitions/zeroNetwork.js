"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zeroNetwork = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.zeroNetwork = (0, defineChain_js_1.defineChain)({
    id: 543_210,
    name: 'Zero Network',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.zerion.io/v1/zero'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Zero Network Explorer',
            url: 'https://explorer.zero.network',
        },
    },
    testnet: false,
});
//# sourceMappingURL=zeroNetwork.js.map