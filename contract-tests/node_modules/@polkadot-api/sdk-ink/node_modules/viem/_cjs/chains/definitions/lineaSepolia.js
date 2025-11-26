"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lineaSepolia = void 0;
const chainConfig_js_1 = require("../../linea/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lineaSepolia = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 59_141,
    name: 'Linea Sepolia Testnet',
    nativeCurrency: { name: 'Linea Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.sepolia.linea.build'],
            webSocket: ['wss://rpc.sepolia.linea.build'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Etherscan',
            url: 'https://sepolia.lineascan.build',
            apiUrl: 'https://api-sepolia.lineascan.build/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 227427,
        },
        ensRegistry: {
            address: '0x5B2636F0f2137B4aE722C01dd5122D7d3e9541f7',
            blockCreated: 2395094,
        },
        ensUniversalResolver: {
            address: '0x4D41762915F83c76EcaF6776d9b08076aA32b492',
            blockCreated: 17_168_484,
        },
    },
    ensTlds: ['.linea.eth'],
    testnet: true,
});
//# sourceMappingURL=lineaSepolia.js.map