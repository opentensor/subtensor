"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.filterCallsSome = filterCallsSome;
exports.createCallFunction = createCallFunction;
exports.decorateExtrinsics = decorateExtrinsics;
const util_1 = require("@polkadot/util");
const lazy_js_1 = require("../../../create/lazy.js");
const index_js_1 = require("../../util/index.js");
const util_js_1 = require("../util.js");
const createUnchecked_js_1 = require("./createUnchecked.js");
function filterCallsSome({ calls }) {
    return calls.isSome;
}
function createCallFunction(registry, lookup, variant, sectionName, sectionIndex) {
    const { fields, index } = variant;
    const count = fields.length;
    const args = new Array(count);
    for (let i = 0; i < count; i++) {
        const { name, type, typeName } = fields[i];
        args[i] = (0, util_1.objectSpread)({
            name: (0, util_1.stringCamelCase)(name.unwrapOr(`param${i}`)),
            type: (0, index_js_1.getSiName)(lookup, type)
        }, typeName.isSome
            ? { typeName: typeName.unwrap() }
            : null);
    }
    return (0, createUnchecked_js_1.createUnchecked)(registry, sectionName, new Uint8Array([sectionIndex, index.toNumber()]), registry.createTypeUnsafe('FunctionMetadataLatest', [(0, util_1.objectSpread)({ args }, variant)]));
}
/** @internal */
function decorateExtrinsics(registry, { lookup, pallets }, version) {
    const result = {};
    const filtered = pallets.filter(filterCallsSome);
    for (let i = 0, count = filtered.length; i < count; i++) {
        const { calls, index, name } = filtered[i];
        const sectionName = (0, util_1.stringCamelCase)(name);
        const sectionIndex = version >= 12 ? index.toNumber() : i;
        (0, util_1.lazyMethod)(result, sectionName, () => (0, lazy_js_1.lazyVariants)(lookup, calls.unwrap(), util_js_1.objectNameToCamel, (variant) => createCallFunction(registry, lookup, variant, sectionName, sectionIndex)));
    }
    return result;
}
