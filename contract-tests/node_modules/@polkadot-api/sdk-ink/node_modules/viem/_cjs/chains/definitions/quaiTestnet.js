"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.quaiTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.quaiTestnet = (0, defineChain_js_1.defineChain)({
    id: 15000,
    name: 'Quai Network Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Quai',
        symbol: 'QUAI',
    },
    rpcUrls: {
        default: { http: ['https://orchard.rpc.quai.network/cyprus1'] },
    },
    blockExplorers: {
        default: {
            name: 'Orchard Quaiscan',
            url: 'https://orchard.quaiscan.io',
            apiUrl: 'https://orchard.quaiscan.io/api',
        },
    },
    testnet: true,
});
//# sourceMappingURL=quaiTestnet.js.map