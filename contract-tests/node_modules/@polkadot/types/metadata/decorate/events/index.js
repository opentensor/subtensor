import { isCodec, isU8a, lazyMethod, stringCamelCase } from '@polkadot/util';
import { lazyVariants } from '../../../create/lazy.js';
import { variantToMeta } from '../errors/index.js';
import { objectNameToString } from '../util.js';
export function filterEventsSome({ events }) {
    return events.isSome;
}
/** @internal */
export function decorateEvents(registry, { lookup, pallets }, version) {
    const result = {};
    const filtered = pallets.filter(filterEventsSome);
    for (let i = 0, count = filtered.length; i < count; i++) {
        const { events, index, name } = filtered[i];
        const sectionIndex = version >= 12 ? index.toNumber() : i;
        lazyMethod(result, stringCamelCase(name), () => lazyVariants(lookup, events.unwrap(), objectNameToString, (variant) => ({
            // We sprinkle in isCodec & isU8a to ensure we are dealing with the correct objects
            is: (eventRecord) => isCodec(eventRecord) &&
                isU8a(eventRecord.index) &&
                sectionIndex === eventRecord.index[0] &&
                variant.index.eq(eventRecord.index[1]),
            meta: registry.createTypeUnsafe('EventMetadataLatest', [variantToMeta(lookup, variant)])
        })));
    }
    return result;
}
