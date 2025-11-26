"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.arenaz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.arenaz = (0, defineChain_js_1.defineChain)({
    id: 7897,
    name: 'Arena-Z',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.arena-z.gg'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Arena-Z Explorer',
            url: 'https://explorer.arena-z.gg',
            apiUrl: 'https://explorer.arena-z.gg',
        },
    },
});
//# sourceMappingURL=arenaz.js.map