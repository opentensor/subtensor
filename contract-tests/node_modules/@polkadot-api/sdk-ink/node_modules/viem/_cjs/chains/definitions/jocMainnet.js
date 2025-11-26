"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.jocMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.jocMainnet = (0, defineChain_js_1.defineChain)({
    id: 81,
    name: 'Japan Open Chain Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Japan Open Chain Token',
        symbol: 'JOC',
    },
    rpcUrls: {
        default: {
            http: [
                'https://rpc-1.japanopenchain.org:8545',
                'https://rpc-2.japanopenchain.org:8545',
                'https://rpc-3.japanopenchain.org',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Block Explorer',
            url: 'https://explorer.japanopenchain.org',
        },
    },
    testnet: false,
});
//# sourceMappingURL=jocMainnet.js.map