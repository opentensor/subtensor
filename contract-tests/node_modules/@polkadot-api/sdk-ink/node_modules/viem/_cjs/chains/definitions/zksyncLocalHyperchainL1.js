"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zksyncLocalHyperchainL1 = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.zksyncLocalHyperchainL1 = (0, defineChain_js_1.defineChain)({
    id: 9,
    name: 'ZKsync CLI Local Hyperchain L1',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['http://localhost:15045'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Blockscout',
            url: 'http://localhost:15001/',
            apiUrl: 'http://localhost:15001/api/v2',
        },
    },
    testnet: true,
});
//# sourceMappingURL=zksyncLocalHyperchainL1.js.map