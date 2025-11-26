"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.undoL1ToL2Alias = undoL1ToL2Alias;
const index_js_1 = require("../../../utils/index.js");
const address_js_1 = require("../../constants/address.js");
function undoL1ToL2Alias(address) {
    let result = BigInt(address) - BigInt(address_js_1.l1ToL2AliasOffset);
    if (result < 0n)
        result += address_js_1.addressModulo;
    return (0, index_js_1.pad)((0, index_js_1.toHex)(result), { size: 20 });
}
//# sourceMappingURL=undoL1ToL2Alias.js.map