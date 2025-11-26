"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.donatuz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.donatuz = (0, defineChain_js_1.defineChain)({
    id: 42_026,
    name: 'Donatuz',
    nativeCurrency: {
        decimals: 18,
        name: 'Ether',
        symbol: 'ETH',
    },
    rpcUrls: {
        default: { http: ['https://rpc.donatuz.com'] },
    },
    blockExplorers: {
        default: {
            name: 'Donatuz Explorer',
            url: 'https://explorer.donatuz.com',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 0,
        },
    },
});
//# sourceMappingURL=donatuz.js.map