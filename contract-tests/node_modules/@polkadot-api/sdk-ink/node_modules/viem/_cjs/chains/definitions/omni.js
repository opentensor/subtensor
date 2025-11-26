"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.omni = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.omni = (0, defineChain_js_1.defineChain)({
    id: 166,
    name: 'Omni',
    nativeCurrency: { name: 'Omni', symbol: 'OMNI', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet.omni.network'],
            webSocket: ['wss://mainnet.omni.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'OmniScan',
            url: 'https://omniscan.network',
        },
    },
    testnet: false,
});
//# sourceMappingURL=omni.js.map