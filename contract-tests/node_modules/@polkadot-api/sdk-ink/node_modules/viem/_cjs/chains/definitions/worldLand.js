"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.worldLand = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.worldLand = (0, defineChain_js_1.defineChain)({
    id: 103,
    name: 'WorldLand Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'WLC',
        symbol: 'WLC',
    },
    rpcUrls: {
        default: {
            http: ['https://seoul.worldland.foundation'],
        },
    },
    blockExplorers: {
        default: {
            name: 'WorldLand Scan',
            url: 'https://scan.worldland.foundation',
        },
    },
    testnet: false,
});
//# sourceMappingURL=worldLand.js.map