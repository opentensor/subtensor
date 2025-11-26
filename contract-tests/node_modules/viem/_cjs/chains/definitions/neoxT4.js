"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.neoxT4 = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.neoxT4 = (0, defineChain_js_1.defineChain)({
    id: 12227332,
    name: 'Neo X Testnet T4',
    nativeCurrency: { name: 'Gas', symbol: 'GAS', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://testnet.rpc.banelabs.org/'],
        },
    },
    blockExplorers: {
        default: {
            name: 'neox-scan',
            url: 'https://xt4scan.ngd.network',
        },
    },
    testnet: true,
});
//# sourceMappingURL=neoxT4.js.map