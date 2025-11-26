"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoGLD1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoGLD1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_008,
    name: 'Juneo GLD1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo GLD1-Chain',
        symbol: 'GLD1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/GLD1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/8',
            apiUrl: 'https://juneoscan.io/chain/8/api',
        },
    },
});
//# sourceMappingURL=juneoGLD1Chain.js.map