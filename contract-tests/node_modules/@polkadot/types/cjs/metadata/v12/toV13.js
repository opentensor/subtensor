"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.toV13 = toV13;
/**
 * @internal
 **/
function toV13(registry, v12) {
    return registry.createTypeUnsafe('MetadataV13', [v12]);
}
