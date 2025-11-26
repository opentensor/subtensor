import type { StorageEntryTypeLatest } from '../interfaces/metadata/index.js';
import type { SiLookupTypeId } from '../interfaces/scaleInfo/index.js';
import type { InterfaceTypes, Registry } from '../types/index.js';
/** @internal */
export declare function unwrapStorageSi(type: StorageEntryTypeLatest): SiLookupTypeId;
/** @internal */
export declare function unwrapStorageType(registry: Registry, type: StorageEntryTypeLatest, isOptional?: boolean): keyof InterfaceTypes;
