"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.creditCoin3Testnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.creditCoin3Testnet = (0, defineChain_js_1.defineChain)({
    id: 102031,
    name: 'Creditcoin3 Testnet',
    nativeCurrency: { name: 'Creditcoin3 Testnet', symbol: 'TCTC', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.cc3-testnet.creditcoin.network'],
            webSocket: ['wss://rpc.cc3-testnet.creditcoin.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://creditcoin-testnet.blockscout.com',
            apiUrl: 'https://creditcoin-testnet.blockscout.com/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=creditCoin3Testnet.js.map