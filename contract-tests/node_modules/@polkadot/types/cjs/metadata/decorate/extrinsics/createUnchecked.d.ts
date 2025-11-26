import type { Registry } from '@polkadot/types-codec/types';
import type { FunctionMetadataLatest } from '../../../interfaces/index.js';
import type { CallFunction } from '../../../types/index.js';
/** @internal */
export declare function createUnchecked(registry: Registry, section: string, callIndex: Uint8Array, callMetadata: FunctionMetadataLatest): CallFunction;
