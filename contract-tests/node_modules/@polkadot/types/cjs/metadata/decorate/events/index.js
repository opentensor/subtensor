"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.filterEventsSome = filterEventsSome;
exports.decorateEvents = decorateEvents;
const util_1 = require("@polkadot/util");
const lazy_js_1 = require("../../../create/lazy.js");
const index_js_1 = require("../errors/index.js");
const util_js_1 = require("../util.js");
function filterEventsSome({ events }) {
    return events.isSome;
}
/** @internal */
function decorateEvents(registry, { lookup, pallets }, version) {
    const result = {};
    const filtered = pallets.filter(filterEventsSome);
    for (let i = 0, count = filtered.length; i < count; i++) {
        const { events, index, name } = filtered[i];
        const sectionIndex = version >= 12 ? index.toNumber() : i;
        (0, util_1.lazyMethod)(result, (0, util_1.stringCamelCase)(name), () => (0, lazy_js_1.lazyVariants)(lookup, events.unwrap(), util_js_1.objectNameToString, (variant) => ({
            // We sprinkle in isCodec & isU8a to ensure we are dealing with the correct objects
            is: (eventRecord) => (0, util_1.isCodec)(eventRecord) &&
                (0, util_1.isU8a)(eventRecord.index) &&
                sectionIndex === eventRecord.index[0] &&
                variant.index.eq(eventRecord.index[1]),
            meta: registry.createTypeUnsafe('EventMetadataLatest', [(0, index_js_1.variantToMeta)(lookup, variant)])
        })));
    }
    return result;
}
