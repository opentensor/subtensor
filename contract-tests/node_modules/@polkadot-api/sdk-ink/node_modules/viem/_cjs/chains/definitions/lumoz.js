"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lumoz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lumoz = (0, defineChain_js_1.defineChain)({
    id: 96_370,
    name: 'Lumoz',
    nativeCurrency: {
        decimals: 18,
        name: 'Lumoz Token',
        symbol: 'MOZ',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.lumoz.org'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Lumoz Scan',
            url: 'https://scan.lumoz.info',
        },
    },
    testnet: false,
});
//# sourceMappingURL=lumoz.js.map