/** @internal */
export function toV11(registry, { modules }) {
    return registry.createTypeUnsafe('MetadataV11', [{
            // This is new in V11, pass V0 here - something non-existing, telling the API to use
            // the fallback for this information (on-chain detection)
            extrinsic: {
                signedExtensions: [],
                version: 0
            },
            modules
        }]);
}
