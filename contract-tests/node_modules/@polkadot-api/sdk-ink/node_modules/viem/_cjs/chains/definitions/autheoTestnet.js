"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.autheoTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.autheoTestnet = (0, defineChain_js_1.defineChain)({
    id: 785,
    name: 'Autheo Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Autheo',
        symbol: 'THEO',
    },
    rpcUrls: {
        default: {
            http: [
                'https://testnet-rpc1.autheo.com',
                'https://testnet-rpc2.autheo.com',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Autheo Testnet Block Explorer',
            url: 'https://testnet-explorer.autheo.com/',
        },
    },
});
//# sourceMappingURL=autheoTestnet.js.map