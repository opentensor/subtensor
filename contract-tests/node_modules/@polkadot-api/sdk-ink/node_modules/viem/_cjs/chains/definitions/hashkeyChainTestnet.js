"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hashkeyTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.hashkeyTestnet = (0, defineChain_js_1.defineChain)({
    id: 133,
    name: 'HashKey Chain Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'HashKey EcoPoints',
        symbol: 'HSK',
    },
    rpcUrls: {
        default: {
            http: ['https://testnet.hsk.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'HashKey Chain Testnet explorer',
            url: 'https://testnet-explorer.hsk.xyz',
        },
    },
    testnet: true,
});
//# sourceMappingURL=hashkeyChainTestnet.js.map