/**
 * @internal
 **/
export function toV13(registry, v12) {
    return registry.createTypeUnsafe('MetadataV13', [v12]);
}
