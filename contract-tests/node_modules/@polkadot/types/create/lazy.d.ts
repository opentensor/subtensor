import type { SiLookupTypeId, SiVariant } from '../interfaces/index.js';
import type { PortableRegistry } from '../metadata/index.js';
interface TypeHolder {
    type: SiLookupTypeId;
}
export declare function lazyVariants<T>(lookup: PortableRegistry, { type }: TypeHolder, getName: (v: SiVariant) => string, creator: (v: SiVariant) => T): Record<string, T>;
export {};
