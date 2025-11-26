import { objectSpread } from '@polkadot/util';
/**
 * @internal
 **/
export function toV12(registry, { extrinsic, modules }) {
    return registry.createTypeUnsafe('MetadataV12', [{
            extrinsic,
            modules: modules.map((mod) => registry.createTypeUnsafe('ModuleMetadataV12', [objectSpread({}, mod, { index: 255 })]))
        }]);
}
