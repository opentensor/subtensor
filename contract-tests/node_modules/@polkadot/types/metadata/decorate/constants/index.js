import { hexToU8a, lazyMethod, lazyMethods, stringCamelCase } from '@polkadot/util';
import { objectNameToCamel } from '../util.js';
/** @internal */
export function decorateConstants(registry, { pallets }, _version) {
    const result = {};
    for (let i = 0, count = pallets.length; i < count; i++) {
        const { constants, name } = pallets[i];
        if (!constants.isEmpty) {
            lazyMethod(result, stringCamelCase(name), () => lazyMethods({}, constants, (constant) => {
                const codec = registry.createTypeUnsafe(registry.createLookupType(constant.type), [hexToU8a(constant.value.toHex())]);
                // We are casting here since we are assigning to a read-only property
                codec.meta = constant;
                return codec;
            }, objectNameToCamel));
        }
    }
    return result;
}
