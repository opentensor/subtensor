"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.kiiTestnetOro = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.kiiTestnetOro = (0, defineChain_js_1.defineChain)({
    id: 1336,
    name: 'Kii Testnet Oro',
    network: 'kii-testnet-oro',
    nativeCurrency: {
        name: 'Kii',
        symbol: 'KII',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://json-rpc.uno.sentry.testnet.v3.kiivalidator.com'],
        },
    },
    blockExplorers: {
        default: {
            name: 'KiiExplorer',
            url: 'https://explorer.kiichain.io/testnet',
        },
    },
    testnet: true,
});
//# sourceMappingURL=kiiTestnet.js.map