"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.unichain = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.unichain = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 130,
    name: 'Unichain',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    blockTime: 1_000,
    rpcUrls: {
        default: {
            http: ['https://mainnet.unichain.org/'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Uniscan',
            url: 'https://uniscan.xyz',
            apiUrl: 'https://api.uniscan.xyz/api',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 0,
        },
        disputeGameFactory: {
            [sourceId]: {
                address: '0x2F12d621a16e2d3285929C9996f478508951dFe4',
            },
        },
        portal: {
            [sourceId]: {
                address: '0x0bd48f6B86a26D3a217d0Fa6FfE2B491B956A7a2',
            },
        },
        l1StandardBridge: {
            [sourceId]: {
                address: '0x81014F44b0a345033bB2b3B21C7a1A308B35fEeA',
            },
        },
    },
    sourceId,
});
//# sourceMappingURL=unichain.js.map