"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.treasure = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const chainConfig_js_1 = require("../../zksync/chainConfig.js");
exports.treasure = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 61_166,
    name: 'Treasure',
    nativeCurrency: {
        decimals: 18,
        name: 'MAGIC',
        symbol: 'MAGIC',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.treasure.lol'],
            webSocket: ['wss://rpc.treasure.lol/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Treasure Block Explorer',
            url: 'https://treasurescan.io',
        },
    },
    contracts: {
        multicall3: {
            address: '0x2e29fe39496a56856D8698bD43e1dF4D0CE6266a',
            blockCreated: 101,
        },
    },
    testnet: false,
});
//# sourceMappingURL=treasure.js.map