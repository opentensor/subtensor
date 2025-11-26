"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lumiaMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lumiaMainnet = (0, defineChain_js_1.defineChain)({
    id: 994873017,
    name: 'Lumia Mainnet',
    network: 'LumiaMainnet',
    nativeCurrency: { name: 'Lumia', symbol: 'LUMIA', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet-rpc.lumia.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Lumia Explorer',
            url: 'https://explorer.lumia.org/',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 3975939,
        },
    },
    testnet: false,
});
//# sourceMappingURL=lumiaMainnet.js.map