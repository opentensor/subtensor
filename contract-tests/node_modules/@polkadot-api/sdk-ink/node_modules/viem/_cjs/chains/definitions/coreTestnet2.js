"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.coreTestnet2 = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.coreTestnet2 = (0, defineChain_js_1.defineChain)({
    id: 1114,
    name: 'Core Testnet2',
    nativeCurrency: {
        decimals: 18,
        name: 'tCore2',
        symbol: 'TCORE2',
    },
    rpcUrls: {
        default: { http: ['https://rpc.test2.btcs.network'] },
    },
    blockExplorers: {
        default: {
            name: 'Core Testnet2',
            url: 'https://scan.test2.btcs.network',
            apiUrl: 'https://api.test2.btcs.network/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0x3CB285ff3Cd5C7C7e570b1E7DE3De17A0f985e56',
            blockCreated: 3_838_600,
        },
    },
    testnet: true,
});
//# sourceMappingURL=coreTestnet2.js.map