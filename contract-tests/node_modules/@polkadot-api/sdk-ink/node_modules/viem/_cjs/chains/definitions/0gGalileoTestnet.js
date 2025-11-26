"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zeroGGalileoTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.zeroGGalileoTestnet = (0, defineChain_js_1.defineChain)({
    id: 16_601,
    name: '0G Galileo Testnet',
    nativeCurrency: { name: 'A0GI', symbol: 'A0GI', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://evmrpc-testnet.0g.ai'],
        },
    },
    blockExplorers: {
        default: {
            name: '0G BlockChain Explorer',
            url: 'https://chainscan-galileo.0g.ai',
        },
    },
    testnet: true,
});
//# sourceMappingURL=0gGalileoTestnet.js.map