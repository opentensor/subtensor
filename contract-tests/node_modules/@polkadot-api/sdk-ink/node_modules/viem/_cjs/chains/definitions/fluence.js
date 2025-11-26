"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.fluence = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.fluence = (0, defineChain_js_1.defineChain)({
    id: 9_999_999,
    name: 'Fluence',
    nativeCurrency: { name: 'FLT', symbol: 'FLT', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.mainnet.fluence.dev'],
            webSocket: ['wss://ws.mainnet.fluence.dev'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://blockscout.mainnet.fluence.dev',
            apiUrl: 'https://blockscout.mainnet.fluence.dev/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 207583,
        },
    },
});
//# sourceMappingURL=fluence.js.map