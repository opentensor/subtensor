"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.redbellyMainnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.redbellyMainnet = (0, defineChain_js_1.defineChain)({
    id: 151,
    name: 'Redbelly Network Mainnet',
    nativeCurrency: {
        name: 'Redbelly Native Coin',
        symbol: 'RBNT',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: ['https://governors.mainnet.redbelly.network'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Routescan',
            url: 'https://redbelly.routescan.io',
            apiUrl: 'https://api.routescan.io/v2/network/mainnet/evm/151/etherscan/api',
        },
    },
    testnet: false,
});
//# sourceMappingURL=redbellyMainnet.js.map