"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decorateConstants = decorateConstants;
const util_1 = require("@polkadot/util");
const util_js_1 = require("../util.js");
/** @internal */
function decorateConstants(registry, { pallets }, _version) {
    const result = {};
    for (let i = 0, count = pallets.length; i < count; i++) {
        const { constants, name } = pallets[i];
        if (!constants.isEmpty) {
            (0, util_1.lazyMethod)(result, (0, util_1.stringCamelCase)(name), () => (0, util_1.lazyMethods)({}, constants, (constant) => {
                const codec = registry.createTypeUnsafe(registry.createLookupType(constant.type), [(0, util_1.hexToU8a)(constant.value.toHex())]);
                // We are casting here since we are assigning to a read-only property
                codec.meta = constant;
                return codec;
            }, util_js_1.objectNameToCamel));
        }
    }
    return result;
}
