"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.eniTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.eniTestnet = (0, defineChain_js_1.defineChain)({
    id: 6_912_115,
    name: 'ENI Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'ENI Testnet Token',
        symbol: 'ENI',
    },
    rpcUrls: {
        default: { http: ['https://rpc-testnet.eniac.network'] },
    },
    blockExplorers: {
        default: {
            name: 'ENI Testnet Explorer',
            url: 'https://scan-testnet.eniac.network',
        },
    },
    testnet: true,
});
//# sourceMappingURL=eniTestnet.js.map