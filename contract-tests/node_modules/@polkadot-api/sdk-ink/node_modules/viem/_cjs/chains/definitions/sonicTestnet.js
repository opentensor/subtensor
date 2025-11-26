"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sonicTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sonicTestnet = (0, defineChain_js_1.defineChain)({
    id: 64_165,
    name: 'Sonic Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Sonic',
        symbol: 'S',
    },
    rpcUrls: {
        default: { http: ['https://rpc.testnet.soniclabs.com'] },
    },
    blockExplorers: {
        default: {
            name: 'Sonic Testnet Explorer',
            url: 'https://testnet.soniclabs.com/',
        },
    },
    testnet: true,
});
//# sourceMappingURL=sonicTestnet.js.map