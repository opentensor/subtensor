import { isCodec, isU8a, lazyMethod, objectSpread, stringCamelCase } from '@polkadot/util';
import { lazyVariants } from '../../../create/lazy.js';
import { objectNameToString } from '../util.js';
export function variantToMeta(lookup, variant) {
    return objectSpread({ args: variant.fields.map(({ type }) => lookup.getTypeDef(type).type) }, variant);
}
/** @internal */
export function decorateErrors(registry, { lookup, pallets }, version) {
    const result = {};
    for (let i = 0, count = pallets.length; i < count; i++) {
        const { errors, index, name } = pallets[i];
        if (errors.isSome) {
            const sectionIndex = version >= 12 ? index.toNumber() : i;
            lazyMethod(result, stringCamelCase(name), () => lazyVariants(lookup, errors.unwrap(), objectNameToString, (variant) => ({
                // We sprinkle in isCodec & isU8a to ensure we are dealing with the correct objects
                is: (errorMod) => isCodec(errorMod) &&
                    isCodec(errorMod.index) &&
                    errorMod.index.eq(sectionIndex) && (isU8a(errorMod.error)
                    ? errorMod.error[0] === variant.index.toNumber()
                    : isCodec(errorMod.error) && errorMod.error.eq(variant.index)),
                meta: registry.createTypeUnsafe('ErrorMetadataLatest', [variantToMeta(lookup, variant)])
            })));
        }
    }
    return result;
}
