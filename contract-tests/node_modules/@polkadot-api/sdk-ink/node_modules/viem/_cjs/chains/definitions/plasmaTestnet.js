"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.plasmaTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.plasmaTestnet = (0, defineChain_js_1.defineChain)({
    id: 9746,
    name: 'Plasma Testnet',
    nativeCurrency: {
        name: 'Testnet Plasma',
        symbol: 'XPL',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://testnet-rpc.plasma.to'],
        },
    },
    blockExplorers: {
        default: {
            name: 'RouteScan',
            url: 'https://testnet.plasmascan.to',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 0,
        },
    },
    testnet: true,
});
//# sourceMappingURL=plasmaTestnet.js.map