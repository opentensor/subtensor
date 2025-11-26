"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xrplevmDevnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.xrplevmDevnet = (0, defineChain_js_1.defineChain)({
    id: 1440002,
    name: 'XRPL EVM Devnet',
    nativeCurrency: {
        name: 'XRP',
        symbol: 'XRP',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.xrplevm.org/'],
        },
        public: {
            http: ['https://rpc.xrplevm.org/'],
        },
    },
    blockExplorers: {
        default: {
            name: 'XRPLEVM Devnet Explorer',
            url: 'https://explorer.xrplevm.org/',
        },
    },
    contracts: {
        multicall3: {
            address: '0x82Cc144D7d0AD4B1c27cb41420e82b82Ad6e9B31',
            blockCreated: 15237286,
        },
    },
    testnet: true,
});
//# sourceMappingURL=xrplevmDevnet.js.map