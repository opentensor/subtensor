"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ink = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.ink = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 57073,
    name: 'Ink',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: [
                'https://rpc-gel.inkonchain.com',
                'https://rpc-qnd.inkonchain.com',
            ],
            webSocket: [
                'wss://rpc-gel.inkonchain.com',
                'wss://rpc-qnd.inkonchain.com',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://explorer.inkonchain.com',
            apiUrl: 'https://explorer.inkonchain.com/api/v2',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        disputeGameFactory: {
            [sourceId]: {
                address: '0x10d7b35078d3baabb96dd45a9143b94be65b12cd',
            },
        },
        portal: {
            [sourceId]: {
                address: '0x5d66c1782664115999c47c9fa5cd031f495d3e4f',
            },
        },
        l1StandardBridge: {
            [sourceId]: {
                address: '0x88ff1e5b602916615391f55854588efcbb7663f0',
            },
        },
    },
    testnet: false,
    sourceId,
});
//# sourceMappingURL=ink.js.map