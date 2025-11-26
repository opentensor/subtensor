import type { StorageEntry } from '../../../primitive/types.js';
import type { Registry } from '../../../types/index.js';
export interface ManualMetadata {
    docs: string;
    type: string;
}
interface ManualDefinition {
    method: string;
    prefix: string;
    section: string;
}
/** @internal */
export declare function createRuntimeFunction({ method, prefix, section }: ManualDefinition, key: Uint8Array | string, { docs, type }: ManualMetadata): (registry: Registry) => StorageEntry;
export {};
