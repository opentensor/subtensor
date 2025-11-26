"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.creditCoin3Devnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.creditCoin3Devnet = (0, defineChain_js_1.defineChain)({
    id: 102032,
    name: 'Creditcoin Devnet',
    nativeCurrency: { name: 'Devnet CTC', symbol: 'devCTC', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.cc3-devnet.creditcoin.network'],
            webSocket: ['wss://rpc.cc3-devnet.creditcoin.network/ws'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'https://creditcoin-devnet.blockscout.com',
            apiUrl: 'https://creditcoin3-dev.subscan.io',
        },
    },
    testnet: true,
});
//# sourceMappingURL=creditCoin3Devnet.js.map