import type { MetadataV13, MetadataV14 } from '../../interfaces/metadata/index.js';
import type { SiVariant } from '../../interfaces/scaleInfo/index.js';
import type { Registry } from '../../types/index.js';
export interface TypeSpec {
    def: {
        HistoricMetaCompat?: string;
        Tuple?: number[];
        Variant?: {
            variants: SiVariant[];
        };
    };
    path?: string[];
}
/**
 * Convert the Metadata to v14
 * @internal
 **/
export declare function toV14(registry: Registry, v13: MetadataV13, metaVersion: number): MetadataV14;
