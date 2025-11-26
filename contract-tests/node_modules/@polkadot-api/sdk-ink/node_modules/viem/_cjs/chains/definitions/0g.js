"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zeroG = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.zeroG = (0, defineChain_js_1.defineChain)({
    id: 16_600,
    name: '0G Newton Testnet',
    nativeCurrency: { name: 'A0GI', symbol: 'A0GI', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://evmrpc-testnet.0g.ai'],
        },
    },
    blockExplorers: {
        default: {
            name: '0G BlockChain Explorer',
            url: 'https://chainscan-newton.0g.ai',
        },
    },
    testnet: true,
});
//# sourceMappingURL=0g.js.map