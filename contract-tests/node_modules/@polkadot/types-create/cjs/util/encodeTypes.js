"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.paramsNotation = paramsNotation;
exports.encodeTypeDef = encodeTypeDef;
exports.withTypeString = withTypeString;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../types/index.js");
const stringIdentity = (value) => value.toString();
const INFO_WRAP = ['BTreeMap', 'BTreeSet', 'Compact', 'HashMap', 'Option', 'Result', 'Vec'];
function paramsNotation(outer, inner, transform = stringIdentity) {
    return `${outer}${inner
        ? `<${(Array.isArray(inner) ? inner : [inner]).map(transform).join(', ')}>`
        : ''}`;
}
function encodeWithParams(registry, typeDef, outer) {
    const { info, sub } = typeDef;
    switch (info) {
        case index_js_1.TypeDefInfo.BTreeMap:
        case index_js_1.TypeDefInfo.BTreeSet:
        case index_js_1.TypeDefInfo.Compact:
        case index_js_1.TypeDefInfo.HashMap:
        case index_js_1.TypeDefInfo.Linkage:
        case index_js_1.TypeDefInfo.Option:
        case index_js_1.TypeDefInfo.Range:
        case index_js_1.TypeDefInfo.RangeInclusive:
        case index_js_1.TypeDefInfo.Result:
        case index_js_1.TypeDefInfo.Vec:
        case index_js_1.TypeDefInfo.WrapperKeepOpaque:
        case index_js_1.TypeDefInfo.WrapperOpaque:
            return paramsNotation(outer, sub, (p) => encodeTypeDef(registry, p));
    }
    throw new Error(`Unable to encode ${(0, util_1.stringify)(typeDef)} with params`);
}
function encodeSubTypes(registry, sub, asEnum, extra) {
    const names = sub.map(({ name }) => name);
    if (!names.every((n) => !!n)) {
        throw new Error(`Subtypes does not have consistent names, ${names.join(', ')}`);
    }
    const inner = (0, util_1.objectSpread)({}, extra);
    for (let i = 0, count = sub.length; i < count; i++) {
        const def = sub[i];
        if (!def.name) {
            throw new Error(`No name found in ${(0, util_1.stringify)(def)}`);
        }
        inner[def.name] = encodeTypeDef(registry, def);
    }
    return (0, util_1.stringify)(asEnum
        ? { _enum: inner }
        : inner);
}
const encoders = {
    [index_js_1.TypeDefInfo.BTreeMap]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'BTreeMap'),
    [index_js_1.TypeDefInfo.BTreeSet]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'BTreeSet'),
    [index_js_1.TypeDefInfo.Compact]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Compact'),
    [index_js_1.TypeDefInfo.DoNotConstruct]: (registry, { displayName, lookupIndex, lookupName }) => `DoNotConstruct<${lookupName || displayName || ((0, util_1.isUndefined)(lookupIndex) ? 'Unknown' : registry.createLookupType(lookupIndex))}>`,
    [index_js_1.TypeDefInfo.Enum]: (registry, { sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Enum type');
        }
        // c-like enums have all Null entries
        // TODO We need to take the disciminant into account and auto-add empty entries
        return sub.every(({ type }) => type === 'Null')
            ? (0, util_1.stringify)({ _enum: sub.map(({ name }, index) => `${name || `Empty${index}`}`) })
            : encodeSubTypes(registry, sub, true);
    },
    [index_js_1.TypeDefInfo.HashMap]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'HashMap'),
    [index_js_1.TypeDefInfo.Int]: (_registry, { length = 32 }) => `Int<${length}>`,
    [index_js_1.TypeDefInfo.Linkage]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Linkage'),
    [index_js_1.TypeDefInfo.Null]: (_registry, _typeDef) => 'Null',
    [index_js_1.TypeDefInfo.Option]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Option'),
    [index_js_1.TypeDefInfo.Plain]: (_registry, { displayName, type }) => displayName || type,
    [index_js_1.TypeDefInfo.Range]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Range'),
    [index_js_1.TypeDefInfo.RangeInclusive]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'RangeInclusive'),
    [index_js_1.TypeDefInfo.Result]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Result'),
    [index_js_1.TypeDefInfo.Set]: (_registry, { length = 8, sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Set type');
        }
        return (0, util_1.stringify)({
            _set: sub.reduce((all, { index, name }, count) => (0, util_1.objectSpread)(all, { [`${name || `Unknown${index || count}`}`]: index || count }), { _bitLength: length || 8 })
        });
    },
    [index_js_1.TypeDefInfo.Si]: (_registry, { lookupName, type }) => lookupName || type,
    [index_js_1.TypeDefInfo.Struct]: (registry, { alias, sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Struct type');
        }
        return encodeSubTypes(registry, sub, false, alias
            ? {
                _alias: [...alias.entries()].reduce((all, [k, v]) => (0, util_1.objectSpread)(all, { [k]: v }), {})
            }
            : {});
    },
    [index_js_1.TypeDefInfo.Tuple]: (registry, { sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Tuple type');
        }
        return `(${sub.map((type) => encodeTypeDef(registry, type)).join(',')})`;
    },
    [index_js_1.TypeDefInfo.UInt]: (_registry, { length = 32 }) => `UInt<${length}>`,
    [index_js_1.TypeDefInfo.Vec]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Vec'),
    [index_js_1.TypeDefInfo.VecFixed]: (_registry, { length, sub }) => {
        if (!(0, util_1.isNumber)(length) || !sub || Array.isArray(sub)) {
            throw new Error('Unable to encode VecFixed type');
        }
        return `[${sub.type};${length}]`;
    },
    [index_js_1.TypeDefInfo.WrapperKeepOpaque]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'WrapperKeepOpaque'),
    [index_js_1.TypeDefInfo.WrapperOpaque]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'WrapperOpaque')
};
function encodeType(registry, typeDef, withLookup = true) {
    return withLookup && typeDef.lookupName
        ? typeDef.lookupName
        : encoders[typeDef.info](registry, typeDef);
}
function encodeTypeDef(registry, typeDef) {
    // In the case of contracts we do have the unfortunate situation where the displayName would
    // refer to "Option" when it is an option. For these, string it out, only using when actually
    // not a top-level element to be used
    return (typeDef.displayName && !INFO_WRAP.some((i) => typeDef.displayName === i))
        ? typeDef.displayName
        : encodeType(registry, typeDef);
}
function withTypeString(registry, typeDef) {
    return (0, util_1.objectSpread)({}, typeDef, {
        type: encodeType(registry, typeDef, false)
    });
}
