"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractTypes = extractTypes;
const types_create_1 = require("@polkadot/types-create");
function extractSubSingle(_, { sub }) {
    const { lookupName, type } = sub;
    return extractTypes([lookupName || type]);
}
function extractSubArray(_, { sub }) {
    return extractTypes(sub.map(({ lookupName, type }) => lookupName || type));
}
function unhandled(type, { info }) {
    throw new Error(`Unhandled: Unable to create and validate type from ${type} (info=${types_create_1.TypeDefInfo[info]})`);
}
const mapping = {
    [types_create_1.TypeDefInfo.BTreeMap]: extractSubArray,
    [types_create_1.TypeDefInfo.BTreeSet]: extractSubSingle,
    [types_create_1.TypeDefInfo.Compact]: extractSubSingle,
    [types_create_1.TypeDefInfo.DoNotConstruct]: unhandled,
    [types_create_1.TypeDefInfo.Enum]: extractSubArray,
    [types_create_1.TypeDefInfo.HashMap]: extractSubArray,
    [types_create_1.TypeDefInfo.Int]: unhandled,
    [types_create_1.TypeDefInfo.Linkage]: extractSubSingle,
    [types_create_1.TypeDefInfo.Null]: unhandled,
    [types_create_1.TypeDefInfo.Option]: extractSubSingle,
    [types_create_1.TypeDefInfo.Plain]: (_, typeDef) => typeDef.lookupName || typeDef.type,
    [types_create_1.TypeDefInfo.Range]: extractSubSingle,
    [types_create_1.TypeDefInfo.RangeInclusive]: extractSubSingle,
    [types_create_1.TypeDefInfo.Result]: extractSubArray,
    [types_create_1.TypeDefInfo.Set]: extractSubArray,
    [types_create_1.TypeDefInfo.Si]: unhandled,
    [types_create_1.TypeDefInfo.Struct]: extractSubArray,
    [types_create_1.TypeDefInfo.Tuple]: extractSubArray,
    [types_create_1.TypeDefInfo.UInt]: unhandled,
    [types_create_1.TypeDefInfo.Vec]: extractSubSingle,
    [types_create_1.TypeDefInfo.VecFixed]: extractSubSingle,
    [types_create_1.TypeDefInfo.WrapperKeepOpaque]: extractSubSingle,
    [types_create_1.TypeDefInfo.WrapperOpaque]: extractSubSingle
};
/** @internal */
function extractTypes(types) {
    const count = types.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        const type = types[i];
        const typeDef = (0, types_create_1.getTypeDef)(type);
        result[i] = mapping[typeDef.info](type, typeDef);
    }
    return result;
}
