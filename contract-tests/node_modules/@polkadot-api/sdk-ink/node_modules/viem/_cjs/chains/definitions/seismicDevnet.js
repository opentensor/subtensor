"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.seismicDevnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.seismicDevnet = (0, defineChain_js_1.defineChain)({
    id: 5124,
    name: 'Seismic Devnet',
    nativeCurrency: { name: 'Seismic Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://node-2.seismicdev.net/rpc'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Seismic Devnet Explorer',
            url: 'https://explorer-2.seismicdev.net',
        },
    },
    testnet: true,
});
//# sourceMappingURL=seismicDevnet.js.map