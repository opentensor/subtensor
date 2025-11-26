import type { MetadataV15, MetadataV16 } from '../../interfaces/metadata/index.js';
import type { Registry } from '../../types/index.js';
/**
 * Convert the Metadata to v16
 * @internal
 **/
export declare function toV16(registry: Registry, v15: MetadataV15, _: number): MetadataV16;
