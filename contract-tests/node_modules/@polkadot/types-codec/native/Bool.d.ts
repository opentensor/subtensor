import type { HexString } from '@polkadot/util/types';
import type { AnyBool, Codec, Inspect, IU8a, Registry } from '../types/index.js';
/**
 * @name bool
 * @description
 * Representation for a boolean value in the system. It extends the base JS `Boolean` class
 * @noInheritDoc
 */
export declare class bool extends Boolean implements Codec {
    readonly registry: Registry;
    createdAtHash?: IU8a;
    initialU8aLength: number;
    isStorageFallback?: boolean;
    constructor(registry: Registry, value?: bool | AnyBool | Uint8Array | number);
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
    /**
     * @description Checks if the value is an empty value (true when it wraps false/default)
     */
    get isEmpty(): boolean;
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isFalse(): boolean;
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isTrue(): boolean;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @description Returns a hex string representation of the value
     */
    toHex(): HexString;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(): boolean;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): boolean;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(): boolean;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare?: boolean): Uint8Array;
}
