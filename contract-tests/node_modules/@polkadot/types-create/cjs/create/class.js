"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.constructTypeClass = constructTypeClass;
exports.getTypeClass = getTypeClass;
exports.createClassUnsafe = createClassUnsafe;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../types/index.js");
const getTypeDef_js_1 = require("../util/getTypeDef.js");
function getTypeDefType({ lookupName, type }) {
    return lookupName || type;
}
function getSubDefArray(value) {
    if (!Array.isArray(value.sub)) {
        throw new Error(`Expected subtype as TypeDef[] in ${(0, util_1.stringify)(value)}`);
    }
    return value.sub;
}
function getSubDef(value) {
    if (!value.sub || Array.isArray(value.sub)) {
        throw new Error(`Expected subtype as TypeDef in ${(0, util_1.stringify)(value)}`);
    }
    return value.sub;
}
function getSubType(value) {
    return getTypeDefType(getSubDef(value));
}
function getTypeClassMap(value) {
    const subs = getSubDefArray(value);
    const map = {};
    for (let i = 0, count = subs.length; i < count; i++) {
        const sub = subs[i];
        if (!sub.name) {
            throw new Error(`No name found in definition ${(0, util_1.stringify)(sub)}`);
        }
        map[sub.name] = getTypeDefType(sub);
    }
    return map;
}
function getTypeClassArray(value) {
    return getSubDefArray(value).map(getTypeDefType);
}
function createInt(Clazz, { displayName, length }) {
    if (!(0, util_1.isNumber)(length)) {
        throw new Error(`Expected bitLength information for ${displayName || Clazz.constructor.name}<bitLength>`);
    }
    return Clazz.with(length, displayName);
}
function createHashMap(Clazz, value) {
    const [keyType, valueType] = getTypeClassArray(value);
    return Clazz.with(keyType, valueType);
}
function createWithSub(Clazz, value) {
    return Clazz.with(getSubType(value));
}
const infoMapping = {
    [index_js_1.TypeDefInfo.BTreeMap]: (_registry, value) => createHashMap(types_codec_1.BTreeMap, value),
    [index_js_1.TypeDefInfo.BTreeSet]: (_registry, value) => createWithSub(types_codec_1.BTreeSet, value),
    [index_js_1.TypeDefInfo.Compact]: (_registry, value) => createWithSub(types_codec_1.Compact, value),
    [index_js_1.TypeDefInfo.DoNotConstruct]: (_registry, value) => types_codec_1.DoNotConstruct.with(value.displayName || value.type),
    [index_js_1.TypeDefInfo.Enum]: (_registry, value) => {
        const subs = getSubDefArray(value);
        return types_codec_1.Enum.with(subs.every(({ type }) => type === 'Null')
            ? subs.reduce((out, { index, name }, count) => {
                if (!name) {
                    throw new Error('No name found in sub definition');
                }
                out[name] = index || count;
                return out;
            }, {})
            : getTypeClassMap(value));
    },
    [index_js_1.TypeDefInfo.HashMap]: (_registry, value) => createHashMap(types_codec_1.HashMap, value),
    [index_js_1.TypeDefInfo.Int]: (_registry, value) => createInt(types_codec_1.Int, value),
    // We have circular deps between Linkage & Struct
    [index_js_1.TypeDefInfo.Linkage]: (_registry, value) => {
        const type = `Option<${getSubType(value)}>`;
        // eslint-disable-next-line sort-keys
        const Clazz = types_codec_1.Struct.with({ previous: type, next: type });
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
        Clazz.prototype.toRawType = function () {
            // eslint-disable-next-line @typescript-eslint/restrict-template-expressions,@typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-call
            return `Linkage<${this.next.toRawType(true)}>`;
        };
        return Clazz;
    },
    [index_js_1.TypeDefInfo.Null]: (_registry, _value) => types_codec_1.Null,
    [index_js_1.TypeDefInfo.Option]: (_registry, value) => {
        if (!value.sub || Array.isArray(value.sub)) {
            throw new Error('Expected type information for Option');
        }
        // NOTE This is opt-in (unhandled), not by default
        // if (value.sub.type === 'bool') {
        //   return OptionBool;
        // }
        return createWithSub(types_codec_1.Option, value);
    },
    [index_js_1.TypeDefInfo.Plain]: (registry, value) => registry.getOrUnknown(value.type),
    [index_js_1.TypeDefInfo.Range]: (_registry, value) => createWithSub(types_codec_1.Range, value),
    [index_js_1.TypeDefInfo.RangeInclusive]: (_registry, value) => createWithSub(types_codec_1.RangeInclusive, value),
    [index_js_1.TypeDefInfo.Result]: (_registry, value) => {
        const [Ok, Err] = getTypeClassArray(value);
        // eslint-disable-next-line @typescript-eslint/no-use-before-define
        return types_codec_1.Result.with({ Err, Ok });
    },
    [index_js_1.TypeDefInfo.Set]: (_registry, value) => types_codec_1.CodecSet.with(getSubDefArray(value).reduce((result, { index, name }) => {
        if (!name || !(0, util_1.isNumber)(index)) {
            throw new Error('No name found in sub definition');
        }
        result[name] = index;
        return result;
    }, {}), value.length),
    [index_js_1.TypeDefInfo.Si]: (registry, value) => getTypeClass(registry, registry.lookup.getTypeDef(value.type)),
    [index_js_1.TypeDefInfo.Struct]: (_registry, value) => types_codec_1.Struct.with(getTypeClassMap(value), value.alias),
    [index_js_1.TypeDefInfo.Tuple]: (_registry, value) => types_codec_1.Tuple.with(getTypeClassArray(value)),
    [index_js_1.TypeDefInfo.UInt]: (_registry, value) => createInt(types_codec_1.UInt, value),
    [index_js_1.TypeDefInfo.Vec]: (_registry, { sub }) => {
        if (!sub || Array.isArray(sub)) {
            throw new Error('Expected type information for vector');
        }
        return (sub.type === 'u8'
            ? types_codec_1.Bytes
            : types_codec_1.Vec.with(getTypeDefType(sub)));
    },
    [index_js_1.TypeDefInfo.VecFixed]: (_registry, { displayName, length, sub }) => {
        if (!(0, util_1.isNumber)(length) || !sub || Array.isArray(sub)) {
            throw new Error('Expected length & type information for fixed vector');
        }
        return (sub.type === 'u8'
            ? types_codec_1.U8aFixed.with((length * 8), displayName)
            : types_codec_1.VecFixed.with(getTypeDefType(sub), length));
    },
    [index_js_1.TypeDefInfo.WrapperKeepOpaque]: (_registry, value) => createWithSub(types_codec_1.WrapperKeepOpaque, value),
    [index_js_1.TypeDefInfo.WrapperOpaque]: (_registry, value) => createWithSub(types_codec_1.WrapperOpaque, value)
};
function constructTypeClass(registry, typeDef) {
    try {
        const Type = infoMapping[typeDef.info](registry, typeDef);
        if (!Type) {
            throw new Error('No class created');
        }
        // don't clobber any existing
        if (!Type.__fallbackType && typeDef.fallbackType) {
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore ...this is the only place we we actually assign this...
            Type.__fallbackType = typeDef.fallbackType;
        }
        return Type;
    }
    catch (error) {
        throw new Error(`Unable to construct class from ${(0, util_1.stringify)(typeDef)}: ${error.message}`);
    }
}
function getTypeClass(registry, typeDef) {
    return registry.getUnsafe(typeDef.type, false, typeDef);
}
function createClassUnsafe(registry, type) {
    return (
    // just retrieve via name, no creation via typeDef
    registry.getUnsafe(type) ||
        // we don't have an existing type, create the class via typeDef
        getTypeClass(registry, registry.isLookupType(type)
            ? registry.lookup.getTypeDef(type)
            : (0, getTypeDef_js_1.getTypeDef)(type)));
}
