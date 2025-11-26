"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.planq = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.planq = (0, defineChain_js_1.defineChain)({
    id: 7070,
    name: 'Planq Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'PLQ',
        symbol: 'PLQ',
    },
    rpcUrls: {
        default: { http: ['https://evm-rpc.planq.network'] },
    },
    blockExplorers: {
        default: {
            name: 'Planq Explorer',
            url: 'https://evm.planq.network',
        },
    },
    testnet: false,
});
//# sourceMappingURL=planq.js.map