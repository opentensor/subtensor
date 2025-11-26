import type { Registry } from '@polkadot/types-codec/types';
import type { DecoratedMeta } from './types.js';
import { Metadata } from '../Metadata.js';
import { decorateConstants } from './constants/index.js';
import { decorateErrors } from './errors/index.js';
import { decorateEvents, filterEventsSome } from './events/index.js';
import { decorateExtrinsics, filterCallsSome } from './extrinsics/index.js';
import { decorateStorage } from './storage/index.js';
/**
 * Expands the metadata by decoration into consts, query and tx sections
 */
export declare function expandMetadata(registry: Registry, metadata: Metadata): DecoratedMeta;
export { decorateConstants, decorateErrors, decorateEvents, decorateExtrinsics, decorateStorage, filterCallsSome, filterEventsSome };
