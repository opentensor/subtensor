"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.neoxMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.neoxMainnet = (0, defineChain_js_1.defineChain)({
    id: 47763,
    name: 'Neo X Mainnet',
    nativeCurrency: { name: 'Gas', symbol: 'GAS', decimals: 18 },
    rpcUrls: {
        default: {
            http: [
                'https://mainnet-1.rpc.banelabs.org',
                'https://mainnet-2.rpc.banelabs.org',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Neo X - Explorer',
            url: 'https://xexplorer.neo.org',
        },
    },
    testnet: false,
});
//# sourceMappingURL=neoxMainnet.js.map