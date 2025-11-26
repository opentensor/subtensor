"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NO_RAW_ARGS = void 0;
exports.createKeyRawParts = createKeyRawParts;
exports.createKeyInspect = createKeyInspect;
exports.createKeyRaw = createKeyRaw;
exports.createFunction = createFunction;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const index_js_1 = require("../../util/index.js");
const getHasher_js_1 = require("./getHasher.js");
exports.NO_RAW_ARGS = {
    args: [],
    hashers: [],
    keys: []
};
/** @internal */
function filterDefined(a) {
    return !(0, util_1.isUndefined)(a);
}
/** @internal */
function assertArgs({ method, section }, { args, keys }) {
    if (!Array.isArray(args)) {
        throw new Error(`Call to ${(0, util_1.stringCamelCase)(section || 'unknown')}.${(0, util_1.stringCamelCase)(method || 'unknown')} needs ${keys.length} arguments`);
    }
    else if (args.filter(filterDefined).length !== keys.length) {
        throw new Error(`Call to ${(0, util_1.stringCamelCase)(section || 'unknown')}.${(0, util_1.stringCamelCase)(method || 'unknown')} needs ${keys.length} arguments, found [${args.join(', ')}]`);
    }
}
/** @internal */
function createKeyRawParts(registry, itemFn, { args, hashers, keys }) {
    const count = keys.length;
    const extra = new Array(count);
    for (let i = 0; i < count; i++) {
        extra[i] = (0, getHasher_js_1.getHasher)(hashers[i])(registry.createTypeUnsafe(registry.createLookupType(keys[i]), [args[i]]).toU8a());
    }
    return [
        [
            (0, util_crypto_1.xxhashAsU8a)(itemFn.prefix, 128),
            (0, util_crypto_1.xxhashAsU8a)(itemFn.method, 128)
        ],
        extra
    ];
}
/** @internal */
function createKeyInspect(registry, itemFn, args) {
    assertArgs(itemFn, args);
    const { meta } = itemFn;
    const [prefix, extra] = createKeyRawParts(registry, itemFn, args);
    let types = [];
    if (meta.type.isMap) {
        const { hashers, key } = meta.type.asMap;
        types = hashers.length === 1
            ? [`${hashers[0].type}(${(0, index_js_1.getSiName)(registry.lookup, key)})`]
            : registry.lookup.getSiType(key).def.asTuple.map((k, i) => `${hashers[i].type}(${(0, index_js_1.getSiName)(registry.lookup, k)})`);
    }
    const names = ['module', 'method'].concat(...args.args.map((_, i) => types[i]));
    return {
        inner: prefix
            .concat(...extra)
            .map((v, i) => ({ name: names[i], outer: [v] }))
    };
}
/** @internal */
function createKeyRaw(registry, itemFn, args) {
    const [prefix, extra] = createKeyRawParts(registry, itemFn, args);
    return (0, util_1.u8aConcat)(...prefix, ...extra);
}
/** @internal */
function createKey(registry, itemFn, args) {
    assertArgs(itemFn, args);
    // always add the length prefix (underlying it is Bytes)
    return (0, util_1.compactAddLength)(createKeyRaw(registry, itemFn, args));
}
/** @internal */
function createStorageInspect(registry, itemFn, options) {
    const { meta: { type } } = itemFn;
    return (...args) => {
        if (type.isPlain) {
            return options.skipHashing
                ? { inner: [], name: 'wellKnown', outer: [(0, util_1.u8aToU8a)(options.key)] }
                : createKeyInspect(registry, itemFn, exports.NO_RAW_ARGS);
        }
        const { hashers, key } = type.asMap;
        return hashers.length === 1
            ? createKeyInspect(registry, itemFn, { args, hashers, keys: [key] })
            : createKeyInspect(registry, itemFn, { args, hashers, keys: registry.lookup.getSiType(key).def.asTuple });
    };
}
/** @internal */
function createStorageFn(registry, itemFn, options) {
    const { meta: { type } } = itemFn;
    let cacheKey = null;
    // Can only have zero or one argument:
    //   - storage.system.account(address)
    //   - storage.timestamp.blockPeriod()
    // For higher-map queries the params are passed in as an tuple, [key1, key2]
    return (...args) => {
        if (type.isPlain) {
            if (!cacheKey) {
                cacheKey = options.skipHashing
                    ? (0, util_1.compactAddLength)((0, util_1.u8aToU8a)(options.key))
                    : createKey(registry, itemFn, exports.NO_RAW_ARGS);
            }
            return cacheKey;
        }
        const { hashers, key } = type.asMap;
        return hashers.length === 1
            ? createKey(registry, itemFn, { args, hashers, keys: [key] })
            : createKey(registry, itemFn, { args, hashers, keys: registry.lookup.getSiType(key).def.asTuple });
    };
}
/** @internal */
function createWithMeta(registry, itemFn, options) {
    const { meta, method, prefix, section } = itemFn;
    const storageFn = createStorageFn(registry, itemFn, options);
    storageFn.inspect = createStorageInspect(registry, itemFn, options);
    storageFn.meta = meta;
    storageFn.method = (0, util_1.stringCamelCase)(method);
    storageFn.prefix = prefix;
    storageFn.section = section;
    // explicitly add the actual method in the toJSON, this gets used to determine caching and without it
    // instances (e.g. collective) will not work since it is only matched on param meta
    storageFn.toJSON = () => (0, util_1.objectSpread)({ storage: { method, prefix, section } }, meta.toJSON());
    return storageFn;
}
/** @internal */
function extendHeadMeta(registry, { meta: { docs, name, type }, section }, { method }, iterFn) {
    // metadata with a fallback value using the type of the key, the normal
    // meta fallback only applies to actual entry values, create one for head
    const meta = registry.createTypeUnsafe('StorageEntryMetadataLatest', [{
            docs,
            fallback: registry.createTypeUnsafe('Bytes', []),
            modifier: registry.createTypeUnsafe('StorageEntryModifierLatest', [1]), // required
            name,
            type: registry.createTypeUnsafe('StorageEntryTypeLatest', [type.asMap.key, 0])
        }]);
    iterFn.meta = meta;
    const fn = (...args) => registry.createTypeUnsafe('StorageKey', [iterFn(...args), { method, section }]);
    fn.meta = meta;
    return fn;
}
/** @internal */
function extendPrefixedMap(registry, itemFn, storageFn) {
    const { meta: { type }, method, section } = itemFn;
    storageFn.iterKey = extendHeadMeta(registry, itemFn, storageFn, (...args) => {
        if (args.length && (type.isPlain || (args.length >= type.asMap.hashers.length))) {
            throw new Error(`Iteration of ${(0, util_1.stringCamelCase)(section || 'unknown')}.${(0, util_1.stringCamelCase)(method || 'unknown')} needs arguments to be at least one less than the full arguments, found [${args.join(', ')}]`);
        }
        if (args.length) {
            if (type.isMap) {
                const { hashers, key } = type.asMap;
                const keysVec = hashers.length === 1
                    ? [key]
                    : registry.lookup.getSiType(key).def.asTuple;
                return new types_codec_1.Raw(registry, createKeyRaw(registry, itemFn, { args, hashers: hashers.slice(0, args.length), keys: keysVec.slice(0, args.length) }));
            }
        }
        return new types_codec_1.Raw(registry, createKeyRaw(registry, itemFn, exports.NO_RAW_ARGS));
    });
    return storageFn;
}
/** @internal */
function createFunction(registry, itemFn, options) {
    const { meta: { type } } = itemFn;
    const storageFn = createWithMeta(registry, itemFn, options);
    if (type.isMap) {
        extendPrefixedMap(registry, itemFn, storageFn);
    }
    storageFn.keyPrefix = (...args) => (storageFn.iterKey && storageFn.iterKey(...args)) ||
        (0, util_1.compactStripLength)(storageFn())[1];
    return storageFn;
}
