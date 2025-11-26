"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.swissdlt = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.swissdlt = (0, defineChain_js_1.defineChain)({
    id: 94,
    name: 'SwissDLT Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'BCTS',
        symbol: 'BCTS',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.swissdlt.ch'],
        },
    },
    blockExplorers: {
        default: {
            name: 'SwissDLT Explorer',
            url: 'https://explorer.swissdlt.ch',
        },
    },
    testnet: false,
});
//# sourceMappingURL=swissdlt.js.map