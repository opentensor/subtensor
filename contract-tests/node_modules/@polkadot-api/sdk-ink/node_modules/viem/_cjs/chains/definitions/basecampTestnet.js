"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.basecampTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.basecampTestnet = (0, defineChain_js_1.defineChain)({
    id: 123420001114,
    name: 'Basecamp Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Camp',
        symbol: 'CAMP',
    },
    rpcUrls: {
        default: { http: ['https://rpc.basecamp.t.raas.gelato.cloud'] },
    },
    blockExplorers: {
        default: {
            name: 'basecamp',
            url: 'https://basecamp.cloud.blockscout.com',
        },
    },
    testnet: true,
});
//# sourceMappingURL=basecampTestnet.js.map