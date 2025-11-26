"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.zeroGMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.zeroGMainnet = (0, defineChain_js_1.defineChain)({
    id: 16_661,
    name: '0G Mainnet',
    nativeCurrency: { name: '0G', symbol: '0G', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://evmrpc.0g.ai'],
        },
    },
    blockExplorers: {
        default: {
            name: '0G BlockChain Explorer',
            url: 'https://chainscan.0g.ai',
        },
    },
    testnet: false,
});
//# sourceMappingURL=0gMainnet.js.map