"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.pumpfiTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.pumpfiTestnet = (0, defineChain_js_1.defineChain)({
    id: 490_092,
    name: 'Pumpfi Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'PMPT',
        symbol: 'PMPT',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc1testnet.pumpfi.me'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Pumpfi Testnet Scan',
            url: 'https://testnetscan.pumpfi.me',
        },
    },
    testnet: true,
});
//# sourceMappingURL=pumpfiTestnet.js.map