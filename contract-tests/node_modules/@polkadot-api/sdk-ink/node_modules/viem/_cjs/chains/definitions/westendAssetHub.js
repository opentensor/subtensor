"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.westendAssetHub = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.westendAssetHub = (0, defineChain_js_1.defineChain)({
    id: 420_420_421,
    name: 'Westend Asset Hub',
    nativeCurrency: {
        decimals: 18,
        name: 'Westies',
        symbol: 'WND',
    },
    rpcUrls: {
        default: { http: ['https://westend-asset-hub-eth-rpc.polkadot.io'] },
    },
    blockExplorers: {
        default: {
            name: 'subscan',
            url: 'https://westend-asset-hub-eth-explorer.parity.io',
        },
    },
    testnet: true,
});
//# sourceMappingURL=westendAssetHub.js.map