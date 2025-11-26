"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.gunz = void 0;
const defineChain_js_1 = require("../../utils/chain/defineChain.js");
exports.gunz = (0, defineChain_js_1.defineChain)({
    id: 43_419,
    name: 'Gunz Mainnet',
    nativeCurrency: { name: 'GUN', symbol: 'GUN', decimals: 18 },
    rpcUrls: {
        default: {
            http: [
                'https://rpc.gunzchain.io/ext/bc/2M47TxWHGnhNtq6pM5zPXdATBtuqubxn5EPFgFmEawCQr9WFML/rpc',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'Gunz Explorer',
            url: 'https://gunzscan.io/',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 70502,
        },
    },
});
//# sourceMappingURL=gunz.js.map