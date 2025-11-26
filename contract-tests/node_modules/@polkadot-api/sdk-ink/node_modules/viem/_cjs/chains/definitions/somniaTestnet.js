"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.somniaTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.somniaTestnet = (0, defineChain_js_1.defineChain)({
    id: 50312,
    name: 'Somnia Testnet',
    nativeCurrency: { name: 'STT', symbol: 'STT', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://dream-rpc.somnia.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Somnia Testnet Explorer',
            url: 'https://shannon-explorer.somnia.network/',
            apiUrl: 'https://shannon-explorer.somnia.network/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0x841b8199E6d3Db3C6f264f6C2bd8848b3cA64223',
            blockCreated: 71314235,
        },
    },
    testnet: true,
});
//# sourceMappingURL=somniaTestnet.js.map