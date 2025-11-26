"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.katana = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.katana = (0, defineChain_js_1.defineChain)({
    id: 747474,
    name: 'Katana',
    network: 'katana',
    nativeCurrency: {
        name: 'Ether',
        symbol: 'ETH',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.katana.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'katana explorer',
            url: 'https://explorer.katanarpc.com',
        },
    },
    testnet: false,
});
//# sourceMappingURL=katana.js.map