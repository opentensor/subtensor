"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.coreTestnet1 = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.coreTestnet1 = (0, defineChain_js_1.defineChain)({
    id: 1115,
    name: 'Core Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'tCore',
        symbol: 'TCORE',
    },
    rpcUrls: {
        default: { http: ['https://rpc.test.btcs.network'] },
    },
    blockExplorers: {
        default: {
            name: 'Core Testnet',
            url: 'https://scan.test.btcs.network',
            apiUrl: 'https://api.test.btcs.network/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xCcddF20A1932537123C2E48Bd8e00b108B8f7569',
            blockCreated: 29_350_509,
        },
    },
    testnet: true,
});
//# sourceMappingURL=coreTestnet1.js.map