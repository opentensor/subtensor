import type { Registry } from '@polkadot/types-codec/types';
import type { MetadataV12, MetadataV13 } from '../../interfaces/metadata/index.js';
/**
 * @internal
 **/
export declare function toV13(registry: Registry, v12: MetadataV12): MetadataV13;
