"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.rivalz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.rivalz = (0, defineChain_js_1.defineChain)({
    id: 753,
    name: 'Rivalz',
    nativeCurrency: { decimals: 18, name: 'Ether', symbol: 'ETH' },
    rpcUrls: {
        default: {
            http: ['https://rivalz.calderachain.xyz/http'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Rivalz Caldera Explorer',
            url: 'https://rivalz.calderaexplorer.xyz',
        },
    },
    testnet: false,
});
//# sourceMappingURL=rivalz.js.map