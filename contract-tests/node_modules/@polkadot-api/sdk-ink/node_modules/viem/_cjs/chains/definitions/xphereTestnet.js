"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xphereTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.xphereTestnet = (0, defineChain_js_1.defineChain)({
    id: 1998991,
    name: 'Xphere Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'XPT',
        symbol: 'XPT',
    },
    rpcUrls: {
        default: {
            http: ['http://testnet.x-phere.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Xphere Tamsa Explorer',
            url: 'https://xpt.tamsa.io',
        },
    },
    testnet: true,
});
//# sourceMappingURL=xphereTestnet.js.map