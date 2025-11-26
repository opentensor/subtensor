"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.kromaSepolia = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.kromaSepolia = (0, defineChain_js_1.defineChain)({
    id: 2358,
    name: 'Kroma Sepolia',
    nativeCurrency: { name: 'Sepolia Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://api.sepolia.kroma.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Kroma Sepolia Explorer',
            url: 'https://blockscout.sepolia.kroma.network',
            apiUrl: 'https://blockscout.sepolia.kroma.network/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 8900914,
        },
    },
    testnet: true,
});
//# sourceMappingURL=kromaSepolia.js.map