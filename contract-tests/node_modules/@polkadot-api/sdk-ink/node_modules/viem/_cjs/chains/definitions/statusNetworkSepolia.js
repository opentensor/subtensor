"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.statusSepolia = void 0;
const chainConfig_js_1 = require("../../linea/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.statusSepolia = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 1_660_990_954,
    name: 'Status Network Sepolia',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: {
            http: ['https://public.sepolia.rpc.status.network'],
            webSocket: ['wss://public.sepolia.rpc.status.network/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://sepoliascan.status.network',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 1_578_364,
        },
    },
    testnet: true,
});
//# sourceMappingURL=statusNetworkSepolia.js.map