"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.eni = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.eni = (0, defineChain_js_1.defineChain)({
    id: 173,
    name: 'ENI Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'ENI',
        symbol: 'ENI',
    },
    rpcUrls: {
        default: { http: ['https://rpc.eniac.network'] },
    },
    blockExplorers: {
        default: {
            name: 'ENI Explorer',
            url: 'https://scan.eniac.network',
        },
    },
    testnet: false,
});
//# sourceMappingURL=eni.js.map