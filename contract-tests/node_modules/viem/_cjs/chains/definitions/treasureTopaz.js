"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.treasureTopaz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const chainConfig_js_1 = require("../../zksync/chainConfig.js");
exports.treasureTopaz = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 978_658,
    name: 'Treasure Topaz Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'MAGIC',
        symbol: 'MAGIC',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.topaz.treasure.lol'],
            webSocket: ['wss://rpc.topaz.treasure.lol/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Treasure Topaz Block Explorer',
            url: 'https://topaz.treasurescan.io',
        },
    },
    contracts: {
        multicall3: {
            address: '0xF9cda624FBC7e059355ce98a31693d299FACd963',
            blockCreated: 108112,
        },
    },
    testnet: true,
});
//# sourceMappingURL=treasureTopaz.js.map