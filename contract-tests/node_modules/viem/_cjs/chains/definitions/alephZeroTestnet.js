"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.alephZeroTestnet = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.alephZeroTestnet = (0, defineChain_js_1.defineChain)({
    id: 2039,
    name: 'Aleph Zero Testnet',
    nativeCurrency: { name: 'TZERO', symbol: 'TZERO', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://rpc.alephzero-testnet.gelato.digital'],
            webSocket: ['wss://ws.alephzero-testnet.gelato.digital'],
        },
    },
    blockExplorers: {
        default: {
            name: 'Aleph Zero EVM Testnet explorer',
            url: 'https://evm-explorer-testnet.alephzero.org',
            apiUrl: 'https://evm-explorer-testnet.alephzero.org/api',
        },
    },
    contracts: {
        multicall3: {
            address: '0xca11bde05977b3631167028862be2a173976ca11',
            blockCreated: 2861745,
        },
    },
    testnet: true,
});
//# sourceMappingURL=alephZeroTestnet.js.map