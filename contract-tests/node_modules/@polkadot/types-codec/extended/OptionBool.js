import { isHex, isU8a, u8aToU8a } from '@polkadot/util';
import { Option } from '../base/Option.js';
import { bool as Bool } from '../native/Bool.js';
function decodeU8a(registry, value) {
    // Encoded as -
    //  - 0 = None
    //  - 1 = True
    //  - 2 = False
    return value[0] === 0
        ? null
        : new Bool(registry, value[0] === 1);
}
/**
 * @name OptionBool
 * @description A specific implementation of Option<bool> than allows for single-byte encoding
 */
export class OptionBool extends Option {
    constructor(registry, value) {
        super(registry, Bool, isU8a(value) || isHex(value)
            ? decodeU8a(registry, u8aToU8a(value))
            : value);
        this.initialU8aLength = 1;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return 1 | 0;
    }
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isFalse() {
        return this.isSome
            ? !this.value.valueOf()
            : false;
    }
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isTrue() {
        return this.isSome
            ? this.value.valueOf()
            : false;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return { outer: [this.toU8a()] };
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(isBare) {
        return isBare
            ? 'bool'
            : 'Option<bool>';
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        if (isBare) {
            return super.toU8a(true);
        }
        return this.isSome
            ? new Uint8Array([this.isTrue ? 1 : 2])
            : new Uint8Array([0]);
    }
}
