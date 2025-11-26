"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.soneium = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.soneium = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 1868,
    name: 'Soneium Mainnet',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.soneium.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://soneium.blockscout.com',
            apiUrl: 'https://soneium.blockscout.com/api',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        disputeGameFactory: {
            [sourceId]: {
                address: '0x512a3d2c7a43bd9261d2b8e8c9c70d4bd4d503c0',
            },
        },
        l2OutputOracle: {
            [sourceId]: {
                address: '0x0000000000000000000000000000000000000000',
            },
        },
        portal: {
            [sourceId]: {
                address: '0x88e529a6ccd302c948689cd5156c83d4614fae92',
                blockCreated: 7061266,
            },
        },
        l1StandardBridge: {
            [sourceId]: {
                address: '0xeb9bf100225c214efc3e7c651ebbadcf85177607',
                blockCreated: 7061266,
            },
        },
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 1,
        },
    },
    sourceId,
});
//# sourceMappingURL=soneium.js.map