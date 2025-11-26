import { getSiName } from '../metadata/util/index.js';
/** @internal */
export function unwrapStorageSi(type) {
    return type.isPlain
        ? type.asPlain
        : type.asMap.value;
}
/** @internal */
export function unwrapStorageType(registry, type, isOptional) {
    const outputType = getSiName(registry.lookup, unwrapStorageSi(type));
    return isOptional
        ? `Option<${outputType}>`
        : outputType;
}
