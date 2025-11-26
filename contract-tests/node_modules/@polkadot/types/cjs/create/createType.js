"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createType = createType;
const types_create_1 = require("@polkadot/types-create");
/**
 * Create an instance of a `type` with a given `params`.
 * @param type - A recognizable string representing the type to create an
 * instance from
 * @param params - The value to instantiate the type with
 */
function createType(registry, type, ...params) {
    return (0, types_create_1.createTypeUnsafe)(registry, type, params);
}
