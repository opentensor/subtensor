import type { HexString } from '@polkadot/util/types';
import type { AnyJson, Codec, CodecClass, Inspect, IU8a, Registry } from '../types/index.js';
/**
 * @name DoNotConstruct
 * @description
 * An unknown type that fails on construction with the type info
 */
export declare class DoNotConstruct implements Codec {
    #private;
    readonly registry: Registry;
    createdAtHash?: IU8a;
    isStorageFallback?: boolean;
    constructor(registry: Registry, typeName?: string);
    static with(typeName?: string): CodecClass;
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
    /**
     * @description Checks if the value is an empty value (always true)
     */
    get isEmpty(): boolean;
    /**
     * @description Unimplemented
     */
    eq(): boolean;
    /**
     * @description Unimplemented
     */
    inspect(): Inspect;
    /**
     * @description Unimplemented
     */
    toHex(): HexString;
    /**
     * @description Unimplemented
     */
    toHuman(): AnyJson;
    /**
     * @description Unimplemented
     */
    toJSON(): AnyJson;
    /**
     * @description Unimplemented
     */
    toPrimitive(): AnyJson;
    /**
     * @description Unimplemented
     */
    toRawType(): string;
    /**
     * @description Unimplemented
     */
    toString(): string;
    /**
     * @description Unimplemented
     */
    toU8a(): Uint8Array;
}
