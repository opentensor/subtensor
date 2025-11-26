"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getSiName = getSiName;
function getSiName(lookup, type) {
    const typeDef = lookup.getTypeDef(type);
    return typeDef.lookupName || typeDef.type;
}
