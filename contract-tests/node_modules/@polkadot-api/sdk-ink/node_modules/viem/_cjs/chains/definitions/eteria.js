"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.eteria = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.eteria = (0, defineChain_js_1.defineChain)({
    id: 140,
    name: 'Eteria',
    nativeCurrency: { name: 'Eteria', symbol: 'ERA', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet.eteria.io/v1'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Eteria Explorer',
            url: 'https://explorer.eteria.io',
            apiUrl: 'https://explorer.eteria.io/api',
        },
    },
});
//# sourceMappingURL=eteria.js.map