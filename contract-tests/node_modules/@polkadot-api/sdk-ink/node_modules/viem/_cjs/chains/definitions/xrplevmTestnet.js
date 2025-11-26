"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xrplevmTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.xrplevmTestnet = (0, defineChain_js_1.defineChain)({
    id: 1449000,
    name: 'XRPL EVM Testnet',
    nativeCurrency: {
        name: 'XRP',
        symbol: 'XRP',
        decimals: 18,
    },
    rpcUrls: {
        default: { http: ['https://rpc.testnet.xrplevm.org'] },
    },
    blockExplorers: {
        default: {
            name: 'blockscout',
            url: 'https://explorer.testnet.xrplevm.org',
            apiUrl: 'https://explorer.testnet.xrplevm.org/api/v2',
        },
    },
    contracts: {
        multicall3: {
            address: '0x82Cc144D7d0AD4B1c27cb41420e82b82Ad6e9B31',
            blockCreated: 492302,
        },
    },
    testnet: true,
});
//# sourceMappingURL=xrplevmTestnet.js.map