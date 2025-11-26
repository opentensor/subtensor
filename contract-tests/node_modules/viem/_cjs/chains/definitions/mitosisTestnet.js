"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.mitosisTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.mitosisTestnet = (0, defineChain_js_1.defineChain)({
    id: 124_832,
    name: 'Mitosis Testnet',
    nativeCurrency: { name: 'MITO', symbol: 'MITO', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.testnet.mitosis.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Mitosis testnet explorer',
            url: 'https://testnet.mitosiscan.xyz',
        },
    },
    testnet: true,
});
//# sourceMappingURL=mitosisTestnet.js.map