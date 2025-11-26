"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.skaleCalypso = void 0;
const defineChain_js_1 = require("../../../utils/chain/defineChain.js");
exports.skaleCalypso = (0, defineChain_js_1.defineChain)({
    id: 1_564_830_818,
    name: 'SKALE Calypso Hub',
    nativeCurrency: { name: 'sFUEL', symbol: 'sFUEL', decimals: 18 },
    rpcUrls: {
        default: {
            http: ['https://mainnet.skalenodes.com/v1/honorable-steel-rasalhague'],
            webSocket: [
                'wss://mainnet.skalenodes.com/v1/ws/honorable-steel-rasalhague',
            ],
        },
    },
    blockExplorers: {
        default: {
            name: 'SKALE Explorer',
            url: 'https://honorable-steel-rasalhague.explorer.mainnet.skalenodes.com',
        },
    },
    contracts: {
        multicall3: {
            address: '0xcA11bde05977b3631167028862bE2a173976CA11',
            blockCreated: 3_107_626,
        },
    },
});
//# sourceMappingURL=calypso.js.map