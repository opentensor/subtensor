"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.quai = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.quai = (0, defineChain_js_1.defineChain)({
    id: 9,
    name: 'Quai Network Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Quai',
        symbol: 'QUAI',
    },
    rpcUrls: {
        default: { http: ['https://rpc.quai.network/cyprus1'] },
    },
    blockExplorers: {
        default: {
            name: 'Quaiscan',
            url: 'https://quaiscan.io',
            apiUrl: 'https://quaiscan.io/api',
        },
    },
    testnet: false,
});
//# sourceMappingURL=quai.js.map