import type { MetadataV14, MetadataV15 } from '../../interfaces/metadata/index.js';
import type { Registry } from '../../types/index.js';
/**
 * Convert the Metadata to v15
 * @internal
 **/
export declare function toV15(registry: Registry, v14: MetadataV14, _: number): MetadataV15;
