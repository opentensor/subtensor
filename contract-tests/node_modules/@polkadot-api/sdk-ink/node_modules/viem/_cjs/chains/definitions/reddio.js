"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.reddio = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.reddio = (0, defineChain_js_1.defineChain)({
    id: 50342,
    name: 'Reddio',
    nativeCurrency: { name: 'Reddio', symbol: 'RED', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet.reddio.com/rpc'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://reddio.cloud.blockscout.com',
            apiUrl: 'https://reddio.cloud.blockscout.com/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 848_849,
        },
    },
    testnet: false,
});
//# sourceMappingURL=reddio.js.map