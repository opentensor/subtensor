"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hyperliquidEvmTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.hyperliquidEvmTestnet = (0, defineChain_js_1.defineChain)({
    id: 998,
    name: 'Hyperliquid EVM Testnet',
    nativeCurrency: { name: 'HYPE', symbol: 'HYPE', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.hyperliquid-testnet.xyz/evm'],
        },
    },
    testnet: true,
});
//# sourceMappingURL=hyperliquidEvmTestnet.js.map