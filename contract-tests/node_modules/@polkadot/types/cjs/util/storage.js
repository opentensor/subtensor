"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.unwrapStorageSi = unwrapStorageSi;
exports.unwrapStorageType = unwrapStorageType;
const index_js_1 = require("../metadata/util/index.js");
/** @internal */
function unwrapStorageSi(type) {
    return type.isPlain
        ? type.asPlain
        : type.asMap.value;
}
/** @internal */
function unwrapStorageType(registry, type, isOptional) {
    const outputType = (0, index_js_1.getSiName)(registry.lookup, unwrapStorageSi(type));
    return isOptional
        ? `Option<${outputType}>`
        : outputType;
}
