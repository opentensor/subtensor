import type { HexString } from '@polkadot/util/types';
import type { Codec, Inspect, IU8a, Registry } from '../types/index.js';
/**
 * @name Null
 * @description
 * Implements a type that does not contain anything (apart from `null`)
 */
export declare class Null implements Codec {
    readonly encodedLength = 0;
    readonly isEmpty = true;
    readonly registry: Registry;
    createdAtHash?: IU8a;
    initialU8aLength: number;
    isStorageFallback?: boolean;
    constructor(registry: Registry);
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
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
    toHuman(): null;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): null;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(): null;
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
