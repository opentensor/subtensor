"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.sonicBlazeTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.sonicBlazeTestnet = (0, defineChain_js_1.defineChain)({
    id: 57_054,
    name: 'Sonic Blaze Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Sonic',
        symbol: 'S',
    },
    rpcUrls: {
        default: { http: ['https://rpc.blaze.soniclabs.com'] },
    },
    blockExplorers: {
        default: {
            name: 'Sonic Blaze Testnet Explorer',
            url: 'https://testnet.sonicscan.org',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 1100,
        },
    },
    testnet: true,
});
//# sourceMappingURL=sonicBlazeTestnet.js.map