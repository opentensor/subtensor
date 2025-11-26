"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.premiumBlockTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.premiumBlockTestnet = (0, defineChain_js_1.defineChain)({
    id: 23_023,
    name: 'PremiumBlock Testnet',
    nativeCurrency: { name: 'Premium Block', symbol: 'PBLK', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.premiumblock.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'PremiumBlocks Explorer',
            url: 'https://scan.premiumblock.org',
        },
    },
    testnet: true,
});
//# sourceMappingURL=premiumBlock.js.map