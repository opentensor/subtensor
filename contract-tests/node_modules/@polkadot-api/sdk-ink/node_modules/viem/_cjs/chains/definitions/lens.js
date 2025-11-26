"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lens = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lens = (0, defineChain_js_1.defineChain)({
    id: 232,
    name: 'Lens',
    nativeCurrency: { name: 'GHO', symbol: 'GHO', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.lens.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Lens Block Explorer',
            url: 'https://explorer.lens.xyz',
            apiUrl: 'https://explorer.lens.xyz/api',
        },
    },
});
//# sourceMappingURL=lens.js.map