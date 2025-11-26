import type { AbiItemType, AbiParameter } from '../../abi.js';
import type { StructLookup } from '../types/structs.js';
/**
 * Gets {@link parameterCache} cache key namespaced by {@link type}. This prevents parameters from being accessible to types that don't allow them (e.g. `string indexed foo` not allowed outside of `type: 'event'`).
 * @param param ABI parameter string
 * @param type ABI parameter type
 * @returns Cache key for {@link parameterCache}
 */
export declare function getParameterCacheKey(param: string, type?: AbiItemType | 'struct', structs?: StructLookup): string;
/**
 * Basic cache seeded with common ABI parameter strings.
 *
 * **Note: When seeding more parameters, make sure you benchmark performance. The current number is the ideal balance between performance and having an already existing cache.**
 */
export declare const parameterCache: Map<string, AbiParameter & {
    indexed?: boolean;
}>;
//# sourceMappingURL=cache.d.ts.map