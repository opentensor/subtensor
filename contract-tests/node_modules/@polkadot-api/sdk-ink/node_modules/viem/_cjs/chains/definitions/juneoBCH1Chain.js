"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.juneoBCH1Chain = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.juneoBCH1Chain = (0, defineChain_js_1.defineChain)({
    id: 45_013,
    name: 'Juneo BCH1-Chain',
    nativeCurrency: {
        decimals: 18,
        name: 'Juneo BCH1-Chain',
        symbol: 'BCH1',
    },
    rpcUrls: {
        default: { http: ['https://rpc.juneo-mainnet.network/ext/bc/BCH1/rpc'] },
    },
    blockExplorers: {
        default: {
            name: 'Juneo Scan',
            url: 'https://juneoscan.io/chain/12',
            apiUrl: 'https://juneoscan.io/chain/12/api',
        },
    },
});
//# sourceMappingURL=juneoBCH1Chain.js.map