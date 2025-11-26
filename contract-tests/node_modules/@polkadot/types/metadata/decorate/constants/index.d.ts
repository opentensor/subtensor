import type { Registry } from '@polkadot/types-codec/types';
import type { MetadataLatest } from '../../../interfaces/index.js';
import type { Constants } from '../types.js';
/** @internal */
export declare function decorateConstants(registry: Registry, { pallets }: MetadataLatest, _version: number): Constants;
