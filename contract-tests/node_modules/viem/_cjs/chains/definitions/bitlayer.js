"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bitlayer = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.bitlayer = (0, defineChain_js_1.defineChain)({
    id: 200901,
    name: 'Bitlayer Mainnet',
    nativeCurrency: {
        name: 'BTC',
        symbol: 'BTC',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.bitlayer.org'],
            webSocket: ['wss://ws.bitlayer.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'bitlayer mainnet scan',
            url: 'https://www.btrscan.com',
        },
    },
    contracts: {
        multicall3: {
            address: '0x5B256fE9e993902eCe49D138a5b1162cBb529474',
            blockCreated: 2421963,
        },
    },
});
//# sourceMappingURL=bitlayer.js.map