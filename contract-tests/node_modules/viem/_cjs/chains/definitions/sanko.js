"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sanko = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sanko = (0, defineChain_js_1.defineChain)({
    id: 1996,
    name: 'Sanko',
    nativeCurrency: { name: 'DMT', symbol: 'DMT', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet.sanko.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Sanko Explorer',
            url: 'https://explorer.sanko.xyz',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 37,
        },
    },
    testnet: false,
});
//# sourceMappingURL=sanko.js.map