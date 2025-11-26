"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.initVerse = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.initVerse = (0, defineChain_js_1.defineChain)({
    id: 7_233,
    name: 'InitVerse Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'InitVerse',
        symbol: 'INI',
    },
    rpcUrls: {
        default: { http: ['https://rpc-mainnet.inichain.com'] },
    },
    blockExplorers: {
        default: {
            name: 'InitVerseScan',
            url: 'https://www.iniscan.com',
            apiUrl: 'https://explorer-api.inichain.com/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0x83466BE48A067115FFF91f7b892Ed1726d032e47',
            blockCreated: 2318,
        },
    },
});
//# sourceMappingURL=initVerse.js.map