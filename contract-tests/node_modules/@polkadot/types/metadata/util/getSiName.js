export function getSiName(lookup, type) {
    const typeDef = lookup.getTypeDef(type);
    return typeDef.lookupName || typeDef.type;
}
