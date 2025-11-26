import type { HexString } from '@polkadot/util/types';
import type { AnyFloat, CodecClass, IFloat, Inspect, IU8a, Registry } from '../types/index.js';
interface Options {
    bitLength?: 32 | 64;
}
/**
 * @name Float
 * @description
 * A Codec wrapper for F32 & F64 values. You generally don't want to be using
 * f32/f64 in your runtime, operations on fixed points numbers are preferable. This class
 * was explicitly added since scale-codec has a flag that enables this and it is available
 * in some eth_* RPCs
 */
export declare class Float extends Number implements IFloat {
    #private;
    readonly encodedLength: number;
    readonly registry: Registry;
    createdAtHash?: IU8a;
    initialU8aLength?: number;
    isStorageFallback?: boolean;
    constructor(registry: Registry, value?: AnyFloat, { bitLength }?: Options);
    static with(bitLength: 32 | 64): CodecClass<Float>;
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
    /**
     * @description Returns true if the type wraps an empty/default all-0 value
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
     * @description Returns a hex string representation of the value
     */
    toHex(): HexString;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(): string;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): string;
    /**
     * @description Returns the number representation (Same as valueOf)
     */
    toNumber(): number;
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(): number;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare?: boolean): Uint8Array;
}
export {};
