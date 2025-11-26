"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoDAI1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoDAI1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_004,
    name: 'Juneo DAI1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo DAI1-Chain',
        symbol: 'DAI1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/DAI1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/5',
            apiUrl: 'https://juneoscan.io/chain/5/api',
        },
    },
});
//# sourceMappingURL=juneoDAI1Chain.js.map