"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoLINK1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoLINK1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_014,
    name: 'Juneo LINK1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo LINK1-Chain',
        symbol: 'LINK1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/LINK1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/13',
            apiUrl: 'https://juneoscan.io/chain/13/api',
        },
    },
});
//# sourceMappingURL=juneoLINK1Chain.js.map