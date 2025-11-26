"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getTypeDef = getTypeDef;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../types/index.js");
const typeSplit_js_1 = require("./typeSplit.js");
const KNOWN_INTERNALS = ['_alias', '_fallback'];
function getTypeString(typeOrObj) {
    return (0, util_1.isString)(typeOrObj)
        ? typeOrObj.toString()
        : (0, util_1.stringify)(typeOrObj);
}
function isRustEnum(details) {
    const values = Object.values(details);
    if (values.some((v) => (0, util_1.isNumber)(v))) {
        if (!values.every((v) => (0, util_1.isNumber)(v) && v >= 0 && v <= 255)) {
            throw new Error('Invalid number-indexed enum definition');
        }
        return false;
    }
    return true;
}
function _decodeEnum(value, details, count, fallbackType) {
    value.info = index_js_1.TypeDefInfo.Enum;
    value.fallbackType = fallbackType;
    // not as pretty, but remain compatible with oo7 for both struct and Array types
    if (Array.isArray(details)) {
        value.sub = details.map((name, index) => ({
            index,
            info: index_js_1.TypeDefInfo.Plain,
            name,
            type: 'Null'
        }));
    }
    else if (isRustEnum(details)) {
        value.sub = Object.entries(details).map(([name, typeOrObj], index) => (0, util_1.objectSpread)({}, getTypeDef(getTypeString(typeOrObj || 'Null'), { name }, count), { index }));
    }
    else {
        value.sub = Object.entries(details).map(([name, index]) => ({
            index,
            info: index_js_1.TypeDefInfo.Plain,
            name,
            type: 'Null'
        }));
    }
    return value;
}
function _decodeSet(value, details, fallbackType) {
    value.info = index_js_1.TypeDefInfo.Set;
    value.fallbackType = fallbackType;
    value.length = details._bitLength;
    value.sub = Object
        .entries(details)
        .filter(([name]) => !name.startsWith('_'))
        .map(([name, index]) => ({
        index,
        info: index_js_1.TypeDefInfo.Plain,
        name,
        type: 'Null'
    }));
    return value;
}
function _decodeStruct(value, type, _, count) {
    const parsed = JSON.parse(type);
    const keys = Object.keys(parsed);
    if (parsed._enum) {
        return _decodeEnum(value, parsed._enum, count, parsed._fallback);
    }
    else if (parsed._set) {
        return _decodeSet(value, parsed._set, parsed._fallback);
    }
    value.alias = parsed._alias
        ? new Map(Object.entries(parsed._alias))
        : undefined;
    value.fallbackType = parsed._fallback;
    value.sub = keys
        .filter((name) => !KNOWN_INTERNALS.includes(name))
        .map((name) => getTypeDef(getTypeString(parsed[name]), { name }, count));
    return value;
}
function _decodeFixedVec(value, type, _, count) {
    const max = type.length - 1;
    let index = -1;
    let inner = 0;
    for (let i = 1; (i < max) && (index === -1); i++) {
        switch (type[i]) {
            case ';': {
                if (inner === 0) {
                    index = i;
                }
                break;
            }
            case '[':
            case '(':
            case '<':
                inner++;
                break;
            case ']':
            case ')':
            case '>':
                inner--;
                break;
        }
    }
    if (index === -1) {
        throw new Error(`${type}: Unable to extract location of ';'`);
    }
    const vecType = type.substring(1, index);
    const [strLength, displayName] = type.substring(index + 1, max).split(';');
    const length = parseInt(strLength.trim(), 10);
    if (length > 2048) {
        throw new Error(`${type}: Only support for [Type; <length>], where length <= 2048`);
    }
    value.displayName = displayName;
    value.length = length;
    value.sub = getTypeDef(vecType, {}, count);
    return value;
}
function _decodeTuple(value, _, subType, count) {
    value.sub = subType.length === 0
        ? []
        : (0, typeSplit_js_1.typeSplit)(subType).map((inner) => getTypeDef(inner, {}, count));
    return value;
}
function _decodeAnyInt(value, type, _, clazz) {
    const [strLength, displayName] = type.substring(clazz.length + 1, type.length - 1).split(',');
    const length = parseInt(strLength.trim(), 10);
    if ((length > 8192) || (length % 8)) {
        throw new Error(`${type}: Only support for ${clazz}<bitLength>, where length <= 8192 and a power of 8, found ${length}`);
    }
    value.displayName = displayName;
    value.length = length;
    return value;
}
function _decodeInt(value, type, subType) {
    return _decodeAnyInt(value, type, subType, 'Int');
}
function _decodeUInt(value, type, subType) {
    return _decodeAnyInt(value, type, subType, 'UInt');
}
function _decodeDoNotConstruct(value, type, _) {
    const NAME_LENGTH = 'DoNotConstruct'.length;
    value.displayName = type.substring(NAME_LENGTH + 1, type.length - 1);
    return value;
}
function hasWrapper(type, [start, end]) {
    return (type.startsWith(start)) && (type.slice(-1 * end.length) === end);
}
const nestedExtraction = [
    ['[', ']', index_js_1.TypeDefInfo.VecFixed, _decodeFixedVec],
    ['{', '}', index_js_1.TypeDefInfo.Struct, _decodeStruct],
    ['(', ')', index_js_1.TypeDefInfo.Tuple, _decodeTuple],
    // the inner for these are the same as tuple, multiple values
    ['BTreeMap<', '>', index_js_1.TypeDefInfo.BTreeMap, _decodeTuple],
    ['HashMap<', '>', index_js_1.TypeDefInfo.HashMap, _decodeTuple],
    ['Int<', '>', index_js_1.TypeDefInfo.Int, _decodeInt],
    ['Result<', '>', index_js_1.TypeDefInfo.Result, _decodeTuple],
    ['UInt<', '>', index_js_1.TypeDefInfo.UInt, _decodeUInt],
    ['DoNotConstruct<', '>', index_js_1.TypeDefInfo.DoNotConstruct, _decodeDoNotConstruct]
];
const wrappedExtraction = [
    ['BTreeSet<', '>', index_js_1.TypeDefInfo.BTreeSet],
    ['Compact<', '>', index_js_1.TypeDefInfo.Compact],
    ['Linkage<', '>', index_js_1.TypeDefInfo.Linkage],
    ['Opaque<', '>', index_js_1.TypeDefInfo.WrapperOpaque],
    ['Option<', '>', index_js_1.TypeDefInfo.Option],
    ['Range<', '>', index_js_1.TypeDefInfo.Range],
    ['RangeInclusive<', '>', index_js_1.TypeDefInfo.RangeInclusive],
    ['Vec<', '>', index_js_1.TypeDefInfo.Vec],
    ['WrapperKeepOpaque<', '>', index_js_1.TypeDefInfo.WrapperKeepOpaque],
    ['WrapperOpaque<', '>', index_js_1.TypeDefInfo.WrapperOpaque]
];
function extractSubType(type, [start, end]) {
    return type.substring(start.length, type.length - end.length);
}
function getTypeDef(_type, { displayName, name } = {}, count = 0) {
    // create the type via Type, allowing types to be sanitized
    const type = (0, types_codec_1.sanitize)(_type);
    const value = { displayName, info: index_js_1.TypeDefInfo.Plain, name, type };
    if (++count > 64) {
        throw new Error('getTypeDef: Maximum nested limit reached');
    }
    const nested = nestedExtraction.find((nested) => hasWrapper(type, nested));
    if (nested) {
        value.info = nested[2];
        return nested[3](value, type, extractSubType(type, nested), count);
    }
    const wrapped = wrappedExtraction.find((wrapped) => hasWrapper(type, wrapped));
    if (wrapped) {
        value.info = wrapped[2];
        value.sub = getTypeDef(extractSubType(type, wrapped), {}, count);
    }
    return value;
}
