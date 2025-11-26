"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.toCallsOnly = toCallsOnly;
const util_1 = require("@polkadot/util");
function trimDocs(docs) {
    const strings = docs.map((d) => d.toString().trim());
    const firstEmpty = strings.findIndex((d) => !d.length);
    return firstEmpty === -1
        ? strings
        : strings.slice(0, firstEmpty);
}
/** @internal */
function toCallsOnly(registry, { extrinsic, lookup, pallets }) {
    return registry.createTypeUnsafe('MetadataLatest', [{
            extrinsic,
            lookup: {
                types: lookup.types.map(({ id, type }) => registry.createTypeUnsafe('PortableType', [{
                        id,
                        type: (0, util_1.objectSpread)({}, type, { docs: trimDocs(type.docs) })
                    }]))
            },
            pallets: pallets.map(({ calls, index, name }) => ({
                calls: registry.createTypeUnsafe('Option<PalletCallMetadataLatest>', [calls.unwrapOr(null)]),
                index,
                name
            }))
        }]).toJSON();
}
