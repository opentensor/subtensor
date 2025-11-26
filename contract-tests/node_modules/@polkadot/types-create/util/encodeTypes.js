import { isNumber, isUndefined, objectSpread, stringify } from '@polkadot/util';
import { TypeDefInfo } from '../types/index.js';
const stringIdentity = (value) => value.toString();
const INFO_WRAP = ['BTreeMap', 'BTreeSet', 'Compact', 'HashMap', 'Option', 'Result', 'Vec'];
export function paramsNotation(outer, inner, transform = stringIdentity) {
    return `${outer}${inner
        ? `<${(Array.isArray(inner) ? inner : [inner]).map(transform).join(', ')}>`
        : ''}`;
}
function encodeWithParams(registry, typeDef, outer) {
    const { info, sub } = typeDef;
    switch (info) {
        case TypeDefInfo.BTreeMap:
        case TypeDefInfo.BTreeSet:
        case TypeDefInfo.Compact:
        case TypeDefInfo.HashMap:
        case TypeDefInfo.Linkage:
        case TypeDefInfo.Option:
        case TypeDefInfo.Range:
        case TypeDefInfo.RangeInclusive:
        case TypeDefInfo.Result:
        case TypeDefInfo.Vec:
        case TypeDefInfo.WrapperKeepOpaque:
        case TypeDefInfo.WrapperOpaque:
            return paramsNotation(outer, sub, (p) => encodeTypeDef(registry, p));
    }
    throw new Error(`Unable to encode ${stringify(typeDef)} with params`);
}
function encodeSubTypes(registry, sub, asEnum, extra) {
    const names = sub.map(({ name }) => name);
    if (!names.every((n) => !!n)) {
        throw new Error(`Subtypes does not have consistent names, ${names.join(', ')}`);
    }
    const inner = objectSpread({}, extra);
    for (let i = 0, count = sub.length; i < count; i++) {
        const def = sub[i];
        if (!def.name) {
            throw new Error(`No name found in ${stringify(def)}`);
        }
        inner[def.name] = encodeTypeDef(registry, def);
    }
    return stringify(asEnum
        ? { _enum: inner }
        : inner);
}
const encoders = {
    [TypeDefInfo.BTreeMap]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'BTreeMap'),
    [TypeDefInfo.BTreeSet]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'BTreeSet'),
    [TypeDefInfo.Compact]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Compact'),
    [TypeDefInfo.DoNotConstruct]: (registry, { displayName, lookupIndex, lookupName }) => `DoNotConstruct<${lookupName || displayName || (isUndefined(lookupIndex) ? 'Unknown' : registry.createLookupType(lookupIndex))}>`,
    [TypeDefInfo.Enum]: (registry, { sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Enum type');
        }
        // c-like enums have all Null entries
        // TODO We need to take the disciminant into account and auto-add empty entries
        return sub.every(({ type }) => type === 'Null')
            ? stringify({ _enum: sub.map(({ name }, index) => `${name || `Empty${index}`}`) })
            : encodeSubTypes(registry, sub, true);
    },
    [TypeDefInfo.HashMap]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'HashMap'),
    [TypeDefInfo.Int]: (_registry, { length = 32 }) => `Int<${length}>`,
    [TypeDefInfo.Linkage]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Linkage'),
    [TypeDefInfo.Null]: (_registry, _typeDef) => 'Null',
    [TypeDefInfo.Option]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Option'),
    [TypeDefInfo.Plain]: (_registry, { displayName, type }) => displayName || type,
    [TypeDefInfo.Range]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Range'),
    [TypeDefInfo.RangeInclusive]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'RangeInclusive'),
    [TypeDefInfo.Result]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Result'),
    [TypeDefInfo.Set]: (_registry, { length = 8, sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Set type');
        }
        return stringify({
            _set: sub.reduce((all, { index, name }, count) => objectSpread(all, { [`${name || `Unknown${index || count}`}`]: index || count }), { _bitLength: length || 8 })
        });
    },
    [TypeDefInfo.Si]: (_registry, { lookupName, type }) => lookupName || type,
    [TypeDefInfo.Struct]: (registry, { alias, sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Struct type');
        }
        return encodeSubTypes(registry, sub, false, alias
            ? {
                _alias: [...alias.entries()].reduce((all, [k, v]) => objectSpread(all, { [k]: v }), {})
            }
            : {});
    },
    [TypeDefInfo.Tuple]: (registry, { sub }) => {
        if (!Array.isArray(sub)) {
            throw new Error('Unable to encode Tuple type');
        }
        return `(${sub.map((type) => encodeTypeDef(registry, type)).join(',')})`;
    },
    [TypeDefInfo.UInt]: (_registry, { length = 32 }) => `UInt<${length}>`,
    [TypeDefInfo.Vec]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'Vec'),
    [TypeDefInfo.VecFixed]: (_registry, { length, sub }) => {
        if (!isNumber(length) || !sub || Array.isArray(sub)) {
            throw new Error('Unable to encode VecFixed type');
        }
        return `[${sub.type};${length}]`;
    },
    [TypeDefInfo.WrapperKeepOpaque]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'WrapperKeepOpaque'),
    [TypeDefInfo.WrapperOpaque]: (registry, typeDef) => encodeWithParams(registry, typeDef, 'WrapperOpaque')
};
function encodeType(registry, typeDef, withLookup = true) {
    return withLookup && typeDef.lookupName
        ? typeDef.lookupName
        : encoders[typeDef.info](registry, typeDef);
}
export function encodeTypeDef(registry, typeDef) {
    // In the case of contracts we do have the unfortunate situation where the displayName would
    // refer to "Option" when it is an option. For these, string it out, only using when actually
    // not a top-level element to be used
    return (typeDef.displayName && !INFO_WRAP.some((i) => typeDef.displayName === i))
        ? typeDef.displayName
        : encodeType(registry, typeDef);
}
export function withTypeString(registry, typeDef) {
    return objectSpread({}, typeDef, {
        type: encodeType(registry, typeDef, false)
    });
}
