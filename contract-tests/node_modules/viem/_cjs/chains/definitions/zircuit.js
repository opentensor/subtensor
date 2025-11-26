"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zircuit = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.zircuit = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 48900,
    name: 'Zircuit Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: [
                'https://zircuit1-mainnet.p2pify.com',
                'https://zircuit1-mainnet.liquify.com',
                'https://zircuit-mainnet.drpc.org',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Zircuit Explorer',
            url: 'https://explorer.zircuit.com',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
        },
        l2OutputOracle: {
            [sourceId]: {
                address: '0x92Ef6Af472b39F1b363da45E35530c24619245A4',
            },
        },
        portal: {
            [sourceId]: {
                address: '0x17bfAfA932d2e23Bd9B909Fd5B4D2e2a27043fb1',
            },
        },
        l1StandardBridge: {
            [sourceId]: {
                address: '0x386B76D9cA5F5Fb150B6BFB35CF5379B22B26dd8',
            },
        },
    },
    testnet: false,
});
//# sourceMappingURL=zircuit.js.map