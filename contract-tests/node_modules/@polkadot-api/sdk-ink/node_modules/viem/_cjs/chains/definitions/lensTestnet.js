"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lensTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lensTestnet = (0, defineChain_js_1.defineChain)({
    id: 37_111,
    name: 'Lens Testnet',
    nativeCurrency: { name: 'GRASS', symbol: 'GRASS', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.lens.dev'],
            webSocket: ['wss://rpc.testnet.lens.dev/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Lens Block Explorer',
            url: 'https://block-explorer.testnet.lens.dev',
            apiUrl: 'https://block-explorer-api.staging.lens.dev/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=lensTestnet.js.map