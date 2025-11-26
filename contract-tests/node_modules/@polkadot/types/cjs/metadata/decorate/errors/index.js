"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.variantToMeta = variantToMeta;
exports.decorateErrors = decorateErrors;
const util_1 = require("@polkadot/util");
const lazy_js_1 = require("../../../create/lazy.js");
const util_js_1 = require("../util.js");
function variantToMeta(lookup, variant) {
    return (0, util_1.objectSpread)({ args: variant.fields.map(({ type }) => lookup.getTypeDef(type).type) }, variant);
}
/** @internal */
function decorateErrors(registry, { lookup, pallets }, version) {
    const result = {};
    for (let i = 0, count = pallets.length; i < count; i++) {
        const { errors, index, name } = pallets[i];
        if (errors.isSome) {
            const sectionIndex = version >= 12 ? index.toNumber() : i;
            (0, util_1.lazyMethod)(result, (0, util_1.stringCamelCase)(name), () => (0, lazy_js_1.lazyVariants)(lookup, errors.unwrap(), util_js_1.objectNameToString, (variant) => ({
                // We sprinkle in isCodec & isU8a to ensure we are dealing with the correct objects
                is: (errorMod) => (0, util_1.isCodec)(errorMod) &&
                    (0, util_1.isCodec)(errorMod.index) &&
                    errorMod.index.eq(sectionIndex) && ((0, util_1.isU8a)(errorMod.error)
                    ? errorMod.error[0] === variant.index.toNumber()
                    : (0, util_1.isCodec)(errorMod.error) && errorMod.error.eq(variant.index)),
                meta: registry.createTypeUnsafe('ErrorMetadataLatest', [variantToMeta(lookup, variant)])
            })));
        }
    }
    return result;
}
