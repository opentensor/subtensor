import type { HexString } from '@polkadot/util/types';
import type { AnyJson, Codec, Inspect, IU8a, IVec, Registry } from '../types/index.js';
/**
 * @name AbstractArray
 * @description
 * This manages codec arrays. It is an extension to Array, providing
 * specific encoding/decoding on top of the base type.
 * @noInheritDoc
 */
export declare abstract class AbstractArray<T extends Codec> extends Array<T> implements IVec<T> {
    readonly registry: Registry;
    createdAtHash?: IU8a;
    initialU8aLength?: number;
    isStorageFallback?: boolean;
    /**
     * @description This ensures that operators such as clice, filter, map, etc. return
     * new Array instances (without this we need to apply overrides)
     */
    static get [Symbol.species](): typeof Array;
    protected constructor(registry: Registry, length: number);
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
    /**
     * @description Checks if the value is an empty value
     */
    get isEmpty(): boolean;
    /**
     * @description The length of the value
     */
    get length(): number;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @internal
     * @description Internal per-item inspection of internal values
     */
    inspectInner(): Inspect[];
    /**
     * @description Converts the Object to an standard JavaScript Array
     */
    toArray(): T[];
    /**
     * @description Returns a hex string representation of the value
     */
    toHex(): HexString;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended?: boolean, disableAscii?: boolean): AnyJson;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): AnyJson;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii?: boolean): AnyJson;
    /**
     * @description Returns the base runtime type name for this instance
     */
    abstract toRawType(): string;
    /**
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare?: boolean): Uint8Array;
    /**
     * @internal
     * @description Internal per-item SCALE encoding of contained values
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8aInner(isBare?: boolean): Uint8Array[];
}
