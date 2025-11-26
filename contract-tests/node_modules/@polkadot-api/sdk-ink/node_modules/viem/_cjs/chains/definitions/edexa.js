"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.edexa = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.edexa = (0, defineChain_js_1.defineChain)({
    id: 5424,
    name: 'edeXa',
    nativeCurrency: { name: 'edeXa', symbol: 'EDX', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.edexa.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'edeXa Explorer',
            url: 'https://explorer.edexa.network',
            apiUrl: 'https://explorer.edexa.network/api/v2',
        },
    },
});
//# sourceMappingURL=edexa.js.map