"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.toV12 = toV12;
const util_1 = require("@polkadot/util");
/**
 * @internal
 **/
function toV12(registry, { extrinsic, modules }) {
    return registry.createTypeUnsafe('MetadataV12', [{
            extrinsic,
            modules: modules.map((mod) => registry.createTypeUnsafe('ModuleMetadataV12', [(0, util_1.objectSpread)({}, mod, { index: 255 })]))
        }]);
}
