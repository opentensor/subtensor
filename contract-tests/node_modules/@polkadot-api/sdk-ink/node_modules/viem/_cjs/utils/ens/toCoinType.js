"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.toCoinType = toCoinType;
const ens_js_1 = require("../../errors/ens.js");
const SLIP44_MSB = 0x80000000;
function toCoinType(chainId) {
    if (chainId === 1)
        return 60n;
    if (chainId >= SLIP44_MSB || chainId < 0)
        throw new ens_js_1.EnsInvalidChainIdError({ chainId });
    return BigInt((0x80000000 | chainId) >>> 0);
}
//# sourceMappingURL=toCoinType.js.map