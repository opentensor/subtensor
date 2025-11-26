"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.dbkchain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.dbkchain = (0, defineChain_js_1.defineChain)({
    id: 20_240_603,
    name: 'DBK chain',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.mainnet.dbkchain.io'],
        },
    },
    blockExplorers: {
        default: {
            name: 'DBK Chain Explorer',
            url: 'https://scan.dbkchain.io',
        },
    },
    testnet: false,
});
//# sourceMappingURL=dbkchain.js.map