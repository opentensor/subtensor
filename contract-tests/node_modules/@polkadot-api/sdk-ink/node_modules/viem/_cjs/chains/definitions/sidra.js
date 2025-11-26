"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sidraChain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sidraChain = (0, defineChain_js_1.defineChain)({
    id: 97_453,
    name: 'Sidra Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Sidra Digital Asset',
        symbol: 'SDA',
    },
    rpcUrls: {
        default: {
            http: ['https://node.sidrachain.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Sidra Chain Explorer',
            url: 'https://ledger.sidrachain.com',
        },
    },
});
//# sourceMappingURL=sidra.js.map