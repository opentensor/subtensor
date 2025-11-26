"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.cornTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 11_155_111;
exports.cornTestnet = (0, defineChain_js_1.defineChain)({
    id: 21_000_001,
    name: 'Corn Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Bitcorn',
        symbol: 'BTCN',
    },
    rpcUrls: {
        default: { http: ['https://rpc.ankr.com/corn_testnet'] },
    },
    blockExplorers: {
        default: {
            name: 'Corn Testnet Explorer',
            url: 'https://testnet.cornscan.io',
            apiUrl: 'https://api.routescan.io/v2/network/testnet/evm/21000001/etherscan/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 4886,
        },
    },
    testnet: true,
    sourceId,
});
//# sourceMappingURL=cornTestnet.js.map