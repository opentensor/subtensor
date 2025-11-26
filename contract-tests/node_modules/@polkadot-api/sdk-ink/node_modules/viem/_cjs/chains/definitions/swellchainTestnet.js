"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.swellchainTestnet = void 0;
const chainConfig_js_1 = require("../../op-stack/chainConfig.js");
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.swellchainTestnet = (0, defineChain_js_1.defineChain)({
    ...chainConfig_js_1.chainConfig,
    id: 1924,
    name: 'Swellchain Testnet',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://swell-testnet.alt.technology'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Swellchain Testnet Explorer',
            url: 'https://swell-testnet-explorer.alt.technology',
            apiUrl: 'https://swell-testnet-explorer.alt.technology/api',
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
//# sourceMappingURL=swellchainTestnet.js.map