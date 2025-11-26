"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.polynomial = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.polynomial = (0, defineChain_js_1.defineChain)({
    id: 8008,
    name: 'Polynomial',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.polynomial.fi'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Polynomial Scan',
            url: 'https://polynomialscan.io',
        },
    },
    testnet: false,
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
        },
    },
});
//# sourceMappingURL=polynomial.js.map