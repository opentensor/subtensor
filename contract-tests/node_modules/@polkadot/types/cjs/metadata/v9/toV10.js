"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.toV10 = toV10;
const util_1 = require("@polkadot/util");
/** @internal */
function createStorageHasher(registry, hasher) {
    // Blake2_128_Concat has been added at index 2, so we increment all the
    // indexes greater than 2
    if (hasher.toNumber() >= 2) {
        return registry.createTypeUnsafe('StorageHasherV10', [hasher.toNumber() + 1]);
    }
    return registry.createTypeUnsafe('StorageHasherV10', [hasher]);
}
/** @internal */
function createStorageType(registry, entryType) {
    if (entryType.isMap) {
        return [(0, util_1.objectSpread)({}, entryType.asMap, {
                hasher: createStorageHasher(registry, entryType.asMap.hasher)
            }), 1];
    }
    if (entryType.isDoubleMap) {
        return [(0, util_1.objectSpread)({}, entryType.asDoubleMap, {
                hasher: createStorageHasher(registry, entryType.asDoubleMap.hasher),
                key2Hasher: createStorageHasher(registry, entryType.asDoubleMap.key2Hasher)
            }), 2];
    }
    return [entryType.asPlain, 0];
}
/** @internal */
function convertModule(registry, mod) {
    const storage = mod.storage.unwrapOr(null);
    return registry.createTypeUnsafe('ModuleMetadataV10', [(0, util_1.objectSpread)({}, mod, {
            storage: storage
                ? (0, util_1.objectSpread)({}, storage, {
                    items: storage.items.map((item) => (0, util_1.objectSpread)({}, item, {
                        type: registry.createTypeUnsafe('StorageEntryTypeV10', createStorageType(registry, item.type))
                    }))
                })
                : null
        })]);
}
/** @internal */
function toV10(registry, { modules }) {
    return registry.createTypeUnsafe('MetadataV10', [{
            modules: modules.map((mod) => convertModule(registry, mod))
        }]);
}
