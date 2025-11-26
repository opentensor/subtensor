"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.saga = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.saga = (0, defineChain_js_1.defineChain)({
    id: 5464,
    name: 'Saga',
    network: 'saga',
    nativeCurrency: {
        decimals: 18,
        name: 'gas',
        symbol: 'GAS',
    },
    rpcUrls: {
        default: { http: ['https://sagaevm.jsonrpc.sagarpc.io'] },
    },
    blockExplorers: {
        default: {
            name: 'Saga Explorer',
            url: 'https://sagaevm.sagaexplorer.io',
        },
    },
    contracts: {
        multicall3: {
            address: '0x864DDc9B50B9A0dF676d826c9B9EDe9F8913a160',
            blockCreated: 467530,
        },
    },
});
//# sourceMappingURL=saga.js.map