"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.swellchain = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.swellchain = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 1923,
    name: 'Swellchain',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: [
                'https://swell-mainnet.alt.technology',
                'https://rpc.ankr.com/swell',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Swell Explorer',
            url: 'https://explorer.swellnetwork.io',
            apiUrl: 'https://explorer.swellnetwork.io/api',
        },
    },
    contracts: {
        ...chainConfig_js_1.chainConfig.contracts,
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 1,
        },
    },
});
//# sourceMappingURL=swellchain.js.map