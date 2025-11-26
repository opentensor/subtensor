"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.memecore = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.memecore = (0, defineChain_js_1.defineChain)({
    id: 4352,
    name: 'MemeCore',
    nativeCurrency: {
        decimals: 18,
        name: 'M',
        symbol: 'M',
    },
    rpcUrls: {
        default: {
            http: ['https://rpc.memecore.net'],
            webSocket: ['wss://ws.memecore.net'],
        },
    },
    blockExplorers: {
        default: {
            name: 'MemeCore Explorer',
            url: 'https://memecorescan.io',
            apiUrl: 'https://api.memecorescan.io/api',
        },
        okx: {
            name: 'MemeCore Explorer',
            url: 'https://web3.okx.com/explorer/memecore',
        },
        memecore: {
            name: 'MemeCore Explorer',
            url: 'https://blockscout.memecore.com',
            apiUrl: 'https://blockscout.memecore.com/api',
        },
    },
});
//# sourceMappingURL=memecore.js.map