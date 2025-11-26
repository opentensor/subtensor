import { lazyMethod } from '@polkadot/util';
export function lazyVariants(lookup, { type }, getName, creator) {
    const result = {};
    const variants = lookup.getSiType(type).def.asVariant.variants;
    for (let i = 0, count = variants.length; i < count; i++) {
        lazyMethod(result, variants[i], creator, getName, i);
    }
    return result;
}
