import type { Registry } from '@polkadot/types-codec/types';
import type { MetadataLatest } from '../../../interfaces/index.js';
import type { Storage } from '../types.js';
/** @internal */
export declare function decorateStorage(registry: Registry, { pallets }: MetadataLatest, _metaVersion: number): Storage;
