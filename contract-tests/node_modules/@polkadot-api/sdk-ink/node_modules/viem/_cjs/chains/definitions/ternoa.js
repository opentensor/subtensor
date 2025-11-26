"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ternoa = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.ternoa = (0, defineChain_js_1.defineChain)({
    id: 752025,
    name: 'Ternoa',
    nativeCurrency: { name: 'Capsule Coin', symbol: 'CAPS', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc-mainnet.zkevm.ternoa.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Ternoa Explorer',
            url: 'https://explorer-mainnet.zkevm.ternoa.network',
        },
    },
    testnet: false,
});
//# sourceMappingURL=ternoa.js.map