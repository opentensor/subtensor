"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.areum = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.areum = (0, defineChain_js_1.defineChain)({
    id: 463,
    name: 'Areum',
    nativeCurrency: { decimals: 18, name: 'AREA', symbol: 'AREA' },
    rpcUrls: {
        default: {
            http: ['https://mainnet-rpc.areum.network'],
            webSocket: ['wss://mainnet-ws.areum.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Areum Explorer',
            url: 'https://explorer.areum.network',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 353286,
        },
    },
    testnet: false,
});
//# sourceMappingURL=areum.js.map