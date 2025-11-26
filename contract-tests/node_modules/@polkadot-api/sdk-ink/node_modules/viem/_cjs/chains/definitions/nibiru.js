"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nibiru = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.nibiru = (0, defineChain_js_1.defineChain)({
    id: 6900,
    name: 'Nibiru',
    nativeCurrency: {
        decimals: 18,
        name: 'NIBI',
        symbol: 'NIBI',
    },
    rpcUrls: {
        default: { http: ['https://evm-rpc.nibiru.fi'] },
    },
    blockExplorers: {
        default: {
            name: 'NibiScan',
            url: 'https://nibiscan.io',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 19587573,
        },
    },
});
//# sourceMappingURL=nibiru.js.map