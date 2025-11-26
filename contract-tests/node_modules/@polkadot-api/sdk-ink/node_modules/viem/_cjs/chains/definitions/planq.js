"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.planq = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.planq = (0, defineChain_js_1.defineChain)({
    id: 7070,
    name: 'Planq Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'PLQ',
        symbol: 'PLQ',
    },
    rpcUrls: {
        default: {
            http: [
                'https://planq-rpc.nodies.app',
                'https://evm-rpc.planq.network',
                'https://jsonrpc.planq.nodestake.top',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Planq Explorer',
            url: 'https://evm.planq.network',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 8470015,
        },
    },
    testnet: false,
});
//# sourceMappingURL=planq.js.map