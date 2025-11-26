"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.celoSepolia = void 0;
const chainConfig_js_1 = require("../../celo/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 11_155_111;
exports.celoSepolia = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 11_142_220,
    name: 'Celo Sepolia Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'CELO',
        symbol: 'S-CELO',
    },
    rpcUrls: {
        default: {
            http: ['https://forno.celo-sepolia.celo-testnet.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Celo Sepolia Explorer',
            url: 'https://celo-sepolia.blockscout.com/',
            apiUrl: 'https://celo-sepolia.blockscout.com/api',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 1,
        },
        portal: {
            [sourceId]: {
                address: '0x44ae3d41a335a7d05eb533029917aad35662dcc2',
                blockCreated: 8825790,
            },
        },
        disputeGameFactory: {
            [sourceId]: {
                address: '0x57c45d82d1a995f1e135b8d7edc0a6bb5211cfaa',
                blockCreated: 8825790,
            },
        },
        l1StandardBridge: {
            [sourceId]: {
                address: '0xec18a3c30131a0db4246e785355fbc16e2eaf408',
                blockCreated: 8825790,
            },
        },
    },
    testnet: true,
});
//# sourceMappingURL=celoSepolia.js.map