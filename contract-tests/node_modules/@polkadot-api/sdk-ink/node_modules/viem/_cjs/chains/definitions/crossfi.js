"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.crossfi = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.crossfi = (0, defineChain_js_1.defineChain)({
    id: 4_158,
    name: 'CrossFi Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'CrossFi',
        symbol: 'XFI',
    },
    rpcUrls: {
        default: { http: ['https://rpc.mainnet.ms'] },
    },
    blockExplorers: {
        default: {
            name: 'CrossFi Blockchain Explorer',
            url: 'https://xfiscan.com',
        },
    },
    testnet: false,
});
//# sourceMappingURL=crossfi.js.map