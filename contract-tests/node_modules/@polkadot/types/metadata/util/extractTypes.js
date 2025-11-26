import { getTypeDef, TypeDefInfo } from '@polkadot/types-create';
function extractSubSingle(_, { sub }) {
    const { lookupName, type } = sub;
    return extractTypes([lookupName || type]);
}
function extractSubArray(_, { sub }) {
    return extractTypes(sub.map(({ lookupName, type }) => lookupName || type));
}
function unhandled(type, { info }) {
    throw new Error(`Unhandled: Unable to create and validate type from ${type} (info=${TypeDefInfo[info]})`);
}
const mapping = {
    [TypeDefInfo.BTreeMap]: extractSubArray,
    [TypeDefInfo.BTreeSet]: extractSubSingle,
    [TypeDefInfo.Compact]: extractSubSingle,
    [TypeDefInfo.DoNotConstruct]: unhandled,
    [TypeDefInfo.Enum]: extractSubArray,
    [TypeDefInfo.HashMap]: extractSubArray,
    [TypeDefInfo.Int]: unhandled,
    [TypeDefInfo.Linkage]: extractSubSingle,
    [TypeDefInfo.Null]: unhandled,
    [TypeDefInfo.Option]: extractSubSingle,
    [TypeDefInfo.Plain]: (_, typeDef) => typeDef.lookupName || typeDef.type,
    [TypeDefInfo.Range]: extractSubSingle,
    [TypeDefInfo.RangeInclusive]: extractSubSingle,
    [TypeDefInfo.Result]: extractSubArray,
    [TypeDefInfo.Set]: extractSubArray,
    [TypeDefInfo.Si]: unhandled,
    [TypeDefInfo.Struct]: extractSubArray,
    [TypeDefInfo.Tuple]: extractSubArray,
    [TypeDefInfo.UInt]: unhandled,
    [TypeDefInfo.Vec]: extractSubSingle,
    [TypeDefInfo.VecFixed]: extractSubSingle,
    [TypeDefInfo.WrapperKeepOpaque]: extractSubSingle,
    [TypeDefInfo.WrapperOpaque]: extractSubSingle
};
/** @internal */
export function extractTypes(types) {
    const count = types.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        const type = types[i];
        const typeDef = getTypeDef(type);
        result[i] = mapping[typeDef.info](type, typeDef);
    }
    return result;
}
