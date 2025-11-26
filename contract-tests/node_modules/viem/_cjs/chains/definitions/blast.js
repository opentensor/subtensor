"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.blast = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
const sourceId = 1;
exports.blast = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 81457,
    name: 'Blast',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: { http: ['https://rpc.blast.io'] },
    },
    blockExplorers: {
        default: {
            name: 'Blastscan',
            url: 'https://blastscan.io',
            apiUrl: 'https://api.blastscan.io/api',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 212929,
        },
    },
    sourceId,
});
//# sourceMappingURL=blast.js.map