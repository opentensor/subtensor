"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericChainProperties = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
function createValue(registry, type, value, asArray = true) {
    // We detect codec here as well - when found, generally this is constructed from itself
    if (value && (0, util_1.isFunction)(value.unwrapOrDefault)) {
        return value;
    }
    return registry.createTypeUnsafe(type, [
        asArray
            ? (0, util_1.isNull)(value) || (0, util_1.isUndefined)(value)
                ? null
                : Array.isArray(value)
                    ? value
                    : [value]
            : value
    ]);
}
function decodeValue(registry, key, value) {
    return key === 'ss58Format'
        ? createValue(registry, 'Option<u32>', value, false)
        : key === 'tokenDecimals'
            ? createValue(registry, 'Option<Vec<u32>>', value)
            : key === 'tokenSymbol'
                ? createValue(registry, 'Option<Vec<Text>>', value)
                : key === 'isEthereum'
                    ? createValue(registry, 'Bool', value, false)
                    : value;
}
function decode(registry, value) {
    return (
    // allow decoding from a map as well (ourselves)
    value && (0, util_1.isFunction)(value.entries)
        ? [...value.entries()]
        : Object.entries(value || {})).reduce((all, [key, value]) => {
        all[key] = decodeValue(registry, key, value);
        return all;
    }, {
        isEthereum: registry.createTypeUnsafe('Bool', []),
        ss58Format: registry.createTypeUnsafe('Option<u32>', []),
        tokenDecimals: registry.createTypeUnsafe('Option<Vec<u32>>', []),
        tokenSymbol: registry.createTypeUnsafe('Option<Vec<Text>>', [])
    });
}
class GenericChainProperties extends types_codec_1.Json {
    constructor(registry, value) {
        super(registry, decode(registry, value));
    }
    /**
     * @description The chain uses Ethereum addresses
     */
    get isEthereum() {
        return this.getT('isEthereum');
    }
    /**
     * @description The chain ss58Format
     */
    get ss58Format() {
        return this.getT('ss58Format');
    }
    /**
     * @description The decimals for each of the tokens
     */
    get tokenDecimals() {
        return this.getT('tokenDecimals');
    }
    /**
     * @description The symbols for the tokens
     */
    get tokenSymbol() {
        return this.getT('tokenSymbol');
    }
}
exports.GenericChainProperties = GenericChainProperties;
