import { objectSpread } from '@polkadot/util';
function trimDocs(docs) {
    const strings = docs.map((d) => d.toString().trim());
    const firstEmpty = strings.findIndex((d) => !d.length);
    return firstEmpty === -1
        ? strings
        : strings.slice(0, firstEmpty);
}
/** @internal */
export function toCallsOnly(registry, { extrinsic, lookup, pallets }) {
    return registry.createTypeUnsafe('MetadataLatest', [{
            extrinsic,
            lookup: {
                types: lookup.types.map(({ id, type }) => registry.createTypeUnsafe('PortableType', [{
                        id,
                        type: objectSpread({}, type, { docs: trimDocs(type.docs) })
                    }]))
            },
            pallets: pallets.map(({ calls, index, name }) => ({
                calls: registry.createTypeUnsafe('Option<PalletCallMetadataLatest>', [calls.unwrapOr(null)]),
                index,
                name
            }))
        }]).toJSON();
}
