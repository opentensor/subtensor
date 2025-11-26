"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jocTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.jocTestnet = (0, defineChain_js_1.defineChain)({
    id: 10081,
    name: 'Japan Open Chain Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Japan Open Chain Testnet Token',
        symbol: 'JOCT',
    },
    rpcUrls: {
        default: {
            http: [
                'https://rpc-1.testnet.japanopenchain.org:8545',
                'https://rpc-2.testnet.japanopenchain.org:8545',
                'https://rpc-3.testnet.japanopenchain.org',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Testnet Block Explorer',
            url: 'https://explorer.testnet.japanopenchain.org',
        },
    },
    testnet: true,
});
//# sourceMappingURL=jocTestnet.js.map