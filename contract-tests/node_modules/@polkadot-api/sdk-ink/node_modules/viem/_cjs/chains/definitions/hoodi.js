"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hoodi = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.hoodi = (0, defineChain_js_1.defineChain)({
    id: 560048,
    name: 'Hoodi',
    nativeCurrency: { name: 'Hoodi Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.hoodi.ethpandaops.io'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Etherscan',
            url: 'https://hoodi.etherscan.io',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 2589,
        },
    },
    testnet: true,
});
//# sourceMappingURL=hoodi.js.map