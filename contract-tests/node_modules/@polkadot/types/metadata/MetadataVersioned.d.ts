import type { AnyJson } from '@polkadot/types-codec/types';
import type { HexString } from '@polkadot/util/types';
import type { MetadataLatest, MetadataV9, MetadataV10, MetadataV11, MetadataV12, MetadataV13, MetadataV14, MetadataV15, MetadataV16 } from '../interfaces/metadata/index.js';
import type { Registry } from '../types/index.js';
import type { MetaVersionAll } from './versions.js';
import { Struct } from '@polkadot/types-codec';
import { MagicNumber } from './MagicNumber.js';
/**
 * @name MetadataVersioned
 * @description
 * The versioned runtime metadata as a decoded structure
 */
export declare class MetadataVersioned extends Struct {
    #private;
    constructor(registry: Registry, value?: Uint8Array | HexString | Map<string, unknown> | Record<string, unknown>);
    /**
     * @description Returns the wrapped metadata as a limited calls-only (latest) version
     */
    get asCallsOnly(): MetadataVersioned;
    /**
     * @description Returns the wrapped metadata as a V9 object
     */
    get asV9(): MetadataV9;
    /**
     * @description Returns the wrapped values as a V10 object
     */
    get asV10(): MetadataV10;
    /**
     * @description Returns the wrapped values as a V11 object
     */
    get asV11(): MetadataV11;
    /**
     * @description Returns the wrapped values as a V12 object
     */
    get asV12(): MetadataV12;
    /**
     * @description Returns the wrapped values as a V13 object
     */
    get asV13(): MetadataV13;
    /**
     * @description Returns the wrapped values as a V14 object
     */
    get asV14(): MetadataV14;
    /**
     * @description Returns the wrapped values as a V15 object
     */
    get asV15(): MetadataV15;
    /**
    * @description Returns the wrapped values as a V16 object
    */
    get asV16(): MetadataV16;
    /**
     * @description Returns the wrapped values as a latest version object
     */
    get asLatest(): MetadataLatest;
    /**
     * @description The magicNumber for the Metadata (known constant)
     */
    get magicNumber(): MagicNumber;
    /**
     * @description the metadata version this structure represents
     */
    get version(): MetaVersionAll;
    getUniqTypes(throwError: boolean): string[];
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): Record<string, AnyJson>;
}
