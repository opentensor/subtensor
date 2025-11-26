"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.monadTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.monadTestnet = (0, defineChain_js_1.defineChain)({
    id: 10_143,
    name: 'Monad Testnet',
    blockTime: 400,
    nativeCurrency: {
        name: 'Testnet MON Token',
        symbol: 'MON',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://testnet-rpc.monad.xyz'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Monad Testnet explorer',
            url: 'https://testnet.monadexplorer.com',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 251449,
        },
    },
    testnet: true,
});
//# sourceMappingURL=monadTestnet.js.map