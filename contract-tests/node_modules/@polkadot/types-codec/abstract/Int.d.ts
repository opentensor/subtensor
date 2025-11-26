import type { HexString } from '@polkadot/util/types';
import type { AnyNumber, Inspect, INumber, IU8a, Registry, UIntBitLength } from '../types/index.js';
import { BN } from '@polkadot/util';
export declare const DEFAULT_UINT_BITS = 64;
/**
 * @name AbstractInt
 * @ignore
 * @noInheritDoc
 */
export declare abstract class AbstractInt extends BN implements INumber {
    #private;
    readonly registry: Registry;
    readonly encodedLength: number;
    readonly isUnsigned: boolean;
    createdAtHash?: IU8a;
    initialU8aLength?: number;
    isStorageFallback?: boolean;
    constructor(registry: Registry, value?: AnyNumber | null, bitLength?: UIntBitLength, isSigned?: boolean);
    /**
     * @description returns a hash of the contents
     */
    get hash(): IU8a;
    /**
     * @description Checks if the value is a zero value (align elsewhere)
     */
    get isEmpty(): boolean;
    /**
     * @description Returns the number of bits in the value
     */
    bitLength(): number;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @description True if this value is the max of the type
     */
    isMax(): boolean;
    /**
     * @description Returns a BigInt representation of the number
     */
    toBigInt(): bigint;
    /**
     * @description Returns the BN representation of the number. (Compatibility)
     */
    toBn(): BN;
    /**
     * @description Returns a hex string representation of the value
     */
    toHex(isLe?: boolean): HexString;
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(_isExpanded?: boolean): string;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(onlyHex?: boolean): any;
    /**
     * @description Returns the value in a primitive form, either number when <= 52 bits, or string otherwise
     */
    toPrimitive(): number | string;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Returns the string representation of the value
     * @param base The base to use for the conversion
     */
    toString(base?: number): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare?: boolean): Uint8Array;
}
