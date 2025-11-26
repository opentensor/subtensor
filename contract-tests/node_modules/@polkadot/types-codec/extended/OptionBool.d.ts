import type { HexString } from '@polkadot/util/types';
import type { AnyBool, Inspect, Registry } from '../types/index.js';
import { Option } from '../base/Option.js';
import { bool as Bool } from '../native/Bool.js';
/**
 * @name OptionBool
 * @description A specific implementation of Option<bool> than allows for single-byte encoding
 */
export declare class OptionBool extends Option<Bool> {
    constructor(registry: Registry, value?: Option<Bool> | AnyBool | Uint8Array | HexString | null);
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isFalse(): boolean;
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isTrue(): boolean;
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(isBare?: boolean): string;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare?: boolean): Uint8Array;
}
