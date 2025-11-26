"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.wmcTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.wmcTestnet = (0, defineChain_js_1.defineChain)({
    id: 42070,
    name: 'WMC Testnet',
    nativeCurrency: { name: 'WMTx', symbol: 'WMTx', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc-testnet-base.worldmobile.net'],
        },
    },
    blockExplorers: {
        default: {
            name: 'WMC Explorer',
            url: 'https://explorer2-base-testnet.worldmobile.net',
        },
    },
    testnet: true,
});
//# sourceMappingURL=wmcTestnet.js.map