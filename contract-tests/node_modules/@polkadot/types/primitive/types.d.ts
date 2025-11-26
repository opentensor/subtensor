import type { StorageEntryMetadataLatest } from '../interfaces/metadata/index.js';
import type { Codec, Inspect } from '../types/index.js';
export interface StorageEntryIterator {
    (...args: unknown[]): Uint8Array & Codec;
    meta: StorageEntryMetadataLatest;
}
export interface StorageEntry {
    (...args: unknown[]): Uint8Array;
    iterKey?: StorageEntryIterator;
    inspect: (...args: unknown[]) => Inspect;
    keyPrefix: (...args: unknown[]) => Uint8Array;
    meta: StorageEntryMetadataLatest;
    method: string;
    prefix: string;
    section: string;
    toJSON: () => any;
}
