"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.corn = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.corn = (0, defineChain_js_1.defineChain)({
    id: 21_000_000,
    name: 'Corn',
    nativeCurrency: {
        decimals: 18,
        name: 'Bitcorn',
        symbol: 'BTCN',
    },
    rpcUrls: {
        default: { http: ['https://rpc.ankr.com/corn_maizenet'] },
    },
    blockExplorers: {
        default: {
            name: 'Corn Explorer',
            url: 'https://cornscan.io',
            apiUrl: 'https://api.routescan.io/v2/network/mainnet/evm/21000000/etherscan/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 3228,
        },
    },
    sourceId,
});
//# sourceMappingURL=corn.js.map