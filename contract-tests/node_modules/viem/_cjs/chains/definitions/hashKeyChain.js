"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hashkey = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.hashkey = (0, defineChain_js_1.defineChain)({
    id: 177,
    name: 'HashKey Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'HashKey EcoPoints',
        symbol: 'HSK',
    },
    rpcUrls: {
        default: {
            http: ['https://mainnet.hsk.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'HashKey Chain Explorer',
            url: 'https://hashkey.blockscout.com',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 0,
        },
    },
});
//# sourceMappingURL=hashKeyChain.js.map