"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.openledger = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.openledger = (0, defineChain_js_1.defineChain)({
    id: 1612,
    name: 'OpenLedger',
    nativeCurrency: { name: 'Open', symbol: 'OPEN', decimals: 18 },
    rpcUrls: {
        default: { http: ['https://rpc.openledger.xyz'] },
    },
    blockExplorers: {
        default: {
            name: 'OpenLedger Explorer',
            url: 'https://scan.openledger.xyz',
        },
    },
    testnet: false,
});
//# sourceMappingURL=openledger.js.map