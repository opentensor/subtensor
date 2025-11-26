"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.b3 = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 8453;
exports.b3 = (0, defineChain_js_1.defineChain)({
    id: 8333,
    name: 'B3',
    nativeCurrency: {
        name: 'Ether',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://mainnet-rpc.b3.fun/http'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://explorer.b3.fun',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 0,
        },
    },
    sourceId,
});
//# sourceMappingURL=b3.js.map