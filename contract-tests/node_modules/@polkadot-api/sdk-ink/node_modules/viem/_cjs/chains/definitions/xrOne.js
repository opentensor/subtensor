"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.xrOne = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.xrOne = (0, defineChain_js_1.defineChain)({
    id: 273,
    name: 'XR One',
    nativeCurrency: {
        decimals: 18,
        name: 'XR1',
        symbol: 'XR1',
    },
    rpcUrls: {
        default: {
            http: ['https://xr1.calderachain.xyz/http'],
            webSocket: ['wss://xr1.calderachain.xyz/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://xr1.calderaexplorer.xyz',
        },
    },
    testnet: false,
});
//# sourceMappingURL=xrOne.js.map