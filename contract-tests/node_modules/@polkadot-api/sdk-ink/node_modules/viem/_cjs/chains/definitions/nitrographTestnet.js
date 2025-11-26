"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nitrographTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.nitrographTestnet = (0, defineChain_js_1.defineChain)({
    id: 200024,
    name: 'Nitrograph Testnet',
    testnet: true,
    rpcUrls: {
        default: {
            http: ['https://rpc-testnet.nitrograph.foundation'],
        },
    },
    nativeCurrency: {
        name: 'Nitro',
        symbol: 'NOS',
        decimals: 18,
    },
    blockExplorers: {
        default: {
            url: 'https://explorer-testnet.nitrograph.foundation',
            name: 'Nitrograph Explorer',
        },
    },
});
//# sourceMappingURL=nitrographTestnet.js.map