"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.pyrope = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 11155111;
exports.pyrope = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    name: 'Pyrope Testnet',
    testnet: true,
    id: 695569,
    sourceId,
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.pyropechain.com'],
            webSocket: ['wss://rpc.pyropechain.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://pyrope.blockscout.com',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        l1StandardBridge: {
            [sourceId]: {
                address: '0xC24932c31D9621aE9e792576152B7ef010cFC2F8',
            },
        },
    },
});
//# sourceMappingURL=pyrope.js.map