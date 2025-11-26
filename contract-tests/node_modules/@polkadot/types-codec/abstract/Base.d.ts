import type { HexString } from '@polkadot/util/types';
import type { AnyJson, BareOpts, Codec, Inspect, IU8a, Registry } from '../types/index.js';
/**
 * @name Base
 * @description A type extends the Base class, when it holds a value
 */
export declare abstract class AbstractBase<T extends Codec> implements Codec {
    #private;
    readonly registry: Registry;
    createdAtHash?: IU8a | undefined;
    initialU8aLength?: number | undefined;
    isStorageFallback?: boolean;
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
     * @description returns the inner (wrapped value)
     */
    get inner(): T;
    /**
     * @description Checks if the value is an empty value
     */
    get isEmpty(): boolean;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @description Returns a hex string representation of the value. isLe returns a LE (number-only) representation
     */
    toHex(isLe?: boolean): HexString;
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
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare?: BareOpts): Uint8Array;
    /**
     * @description Returns the base runtime type name for this instance
     */
    abstract toRawType(): string;
    /**
     * @description Returns the inner wrapped value (equivalent to valueOf)
     */
    unwrap(): T;
    /**
     * @description Returns the inner wrapped value
     */
    valueOf(): T;
}
