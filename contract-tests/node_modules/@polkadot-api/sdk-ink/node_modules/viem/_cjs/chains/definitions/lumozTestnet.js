"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lumozTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.lumozTestnet = (0, defineChain_js_1.defineChain)({
    id: 105_363,
    name: 'Lumoz Testnet',
    nativeCurrency: {
        decimals: 18,
        name: 'Lumoz Testnet Token',
        symbol: 'MOZ',
    },
    rpcUrls: {
        default: {
            http: ['https://testnet-rpc.lumoz.org'],
        },
    },
    testnet: true,
});
//# sourceMappingURL=lumozTestnet.js.map