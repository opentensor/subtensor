"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.plasmaDevnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.plasmaDevnet = (0, defineChain_js_1.defineChain)({
    id: 9747,
    name: 'Plasma Devnet',
    nativeCurrency: {
        name: 'Devnet Plasma',
        symbol: 'XPL',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://devnet-rpc.plasma.to'],
        },
    },
    testnet: true,
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 0,
        },
    },
});
//# sourceMappingURL=plasmaDevnet.js.map