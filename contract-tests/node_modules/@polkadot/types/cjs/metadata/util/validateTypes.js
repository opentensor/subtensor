"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateTypes = validateTypes;
const util_1 = require("@polkadot/util");
const extractTypes_js_1 = require("./extractTypes.js");
const flattenUniq_js_1 = require("./flattenUniq.js");
const l = (0, util_1.logger)('metadata');
/** @internal */
function validateTypes(registry, throwError, types) {
    const missing = (0, flattenUniq_js_1.flattenUniq)((0, extractTypes_js_1.extractTypes)(types))
        .filter((type) => !registry.hasType(type) &&
        !registry.isLookupType(type))
        .sort();
    if (missing.length !== 0) {
        const message = `Unknown types found, no types for ${missing.join(', ')}`;
        if (throwError) {
            throw new Error(message);
        }
        else {
            l.warn(message);
        }
    }
    return types;
}
