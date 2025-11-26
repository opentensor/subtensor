"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.riseTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.riseTestnet = (0, defineChain_js_1.defineChain)({
    id: 11_155_931,
    name: 'RISE Testnet',
    nativeCurrency: { name: 'RISE Testnet Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://testnet.riselabs.xyz'],
            webSocket: ['wss://testnet.riselabs.xyz/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://explorer.testnet.riselabs.xyz/',
            apiUrl: 'https://explorer.testnet.riselabs.xyz/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
        },
    },
    testnet: true,
});
//# sourceMappingURL=riseTestnet.js.map