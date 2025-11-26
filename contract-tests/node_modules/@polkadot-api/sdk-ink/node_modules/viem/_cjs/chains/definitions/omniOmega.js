"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.omniOmega = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.omniOmega = (0, defineChain_js_1.defineChain)({
    id: 164,
    name: 'Omni Omega',
    nativeCurrency: { name: 'Omni', symbol: 'OMNI', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://omega.omni.network'],
            webSocket: ['wss://omega.omni.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Omega OmniScan',
            url: 'https://omega.omniscan.network/',
        },
    },
    testnet: true,
});
//# sourceMappingURL=omniOmega.js.map