"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.creditCoin3Mainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.creditCoin3Mainnet = (0, defineChain_js_1.defineChain)({
    id: 102030,
    name: 'Creditcoin',
    nativeCurrency: { name: 'Creditcoin', symbol: 'CTC', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet3.creditcoin.network'],
            webSocket: ['wss://mainnet3.creditcoin.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://creditcoin.blockscout.com',
            apiUrl: 'https://creditcoin.blockscout.com/api',
        },
    },
    testnet: false,
});
//# sourceMappingURL=creditCoin3Mainnet.js.map