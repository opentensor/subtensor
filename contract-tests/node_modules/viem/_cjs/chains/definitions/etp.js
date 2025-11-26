"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.etp = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.etp = (0, defineChain_js_1.defineChain)({
    id: 20_256_789,
    name: 'ETP Mainnet',
    nativeCurrency: {
        decimals: 18,
        name: 'ETP Chain Native Token',
        symbol: 'ETP',
    },
    rpcUrls: {
        default: { http: ['https://rpc.etpscan.xyz'] },
    },
    blockExplorers: {
        default: {
            name: 'ETP Scan',
            url: 'https://etpscan.xyz',
        },
    },
});
//# sourceMappingURL=etp.js.map