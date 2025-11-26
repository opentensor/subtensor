import type { HexString } from '@polkadot/util/types';
import type { AnyJson, BareOpts, CodecObject, Inspect, IU8a, Registry, ToString } from '../types/index.js';
/**
 * @name Object
 * @description A type extends the Base class, when it holds a value
 */
export declare abstract class AbstractObject<T extends ToString> implements CodecObject<T> {
    readonly registry: Registry;
    createdAtHash?: IU8a;
    initialU8aLength?: number | undefined;
    isStorageFallback?: boolean;
    readonly $: T;
    protected constructor(registry: Registry, value: T, initialU8aLength?: number);
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
    abstract get isEmpty(): boolean;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    abstract eq(other?: unknown): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    abstract inspect(): Inspect;
    /**
     * @description Returns a hex string representation of the value. isLe returns a LE (number-only) representation
     */
    abstract toHex(isLe?: boolean): HexString;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    abstract toHuman(isExtended?: boolean, disableAscii?: boolean): AnyJson;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    abstract toJSON(): AnyJson;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    abstract toPrimitive(disableAscii?: boolean): AnyJson;
    /**
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    abstract toU8a(isBare?: BareOpts): Uint8Array;
    /**
     * @description Returns the base runtime type name for this instance
     */
    abstract toRawType(): string;
    /**
     * @description Return the internal value (JS-aligned, same result as $)
     */
    valueOf(): T;
}
