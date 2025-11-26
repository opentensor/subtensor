"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xphereMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.xphereMainnet = (0, defineChain_js_1.defineChain)({
    id: 20250217,
    name: 'Xphere Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'XP',
        symbol: 'XP',
    },
    rpcUrls: {
        default: {
            http: ['https://en-bkk.x-phere.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Xphere Tamsa Explorer',
            url: 'https://xp.tamsa.io',
        },
    },
    testnet: false,
});
//# sourceMappingURL=xphereMainnet.js.map