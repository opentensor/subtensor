"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.eduChainTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.eduChainTestnet = (0, defineChain_js_1.defineChain)({
    id: 656476,
    name: 'EDU Chain Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'EDU',
        symbol: 'EDU',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.open-campus-codex.gelato.digital/'],
            webSocket: ['wss://ws.open-campus-codex.gelato.digital'],
        },
    },
    blockExplorers: {
        default: {
            name: 'EDU Chain Testnet Explorer',
            url: 'https://opencampus-codex.blockscout.com',
            apiUrl: 'https://opencampus-codex.blockscout.com/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 15514133,
        },
    },
    testnet: true,
});
//# sourceMappingURL=eduChainTestnet.js.map