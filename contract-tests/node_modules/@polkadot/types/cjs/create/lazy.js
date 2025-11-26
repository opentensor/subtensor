"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.lazyVariants = lazyVariants;
const util_1 = require("@polkadot/util");
function lazyVariants(lookup, { type }, getName, creator) {
    const result = {};
    const variants = lookup.getSiType(type).def.asVariant.variants;
    for (let i = 0, count = variants.length; i < count; i++) {
        (0, util_1.lazyMethod)(result, variants[i], creator, getName, i);
    }
    return result;
}
