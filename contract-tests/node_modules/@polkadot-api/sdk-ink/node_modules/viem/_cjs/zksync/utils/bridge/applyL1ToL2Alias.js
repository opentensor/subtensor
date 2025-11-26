"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.applyL1ToL2Alias = applyL1ToL2Alias;
const index_js_1 = require("../../../utils/index.js");
const address_js_1 = require("../../constants/address.js");
function applyL1ToL2Alias(address) {
    return (0, index_js_1.pad)((0, index_js_1.toHex)((BigInt(address) + BigInt(address_js_1.l1ToL2AliasOffset)) % address_js_1.addressModulo), { size: 20 });
}
//# sourceMappingURL=applyL1ToL2Alias.js.map