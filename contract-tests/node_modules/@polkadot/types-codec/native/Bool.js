import { isU8a, u8aToHex } from '@polkadot/util';
/**
 * @name bool
 * @description
 * Representation for a boolean value in the system. It extends the base JS `Boolean` class
 * @noInheritDoc
 */
export class bool extends Boolean {
    registry;
    createdAtHash;
    initialU8aLength = 1;
    isStorageFallback;
    constructor(registry, value = false) {
        super(isU8a(value)
            ? value[0] === 1
            : value instanceof Boolean
                ? value.valueOf()
                : !!value);
        this.registry = registry;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return 1 | 0;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Checks if the value is an empty value (true when it wraps false/default)
     */
    get isEmpty() {
        return this.isFalse;
    }
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isFalse() {
        return !this.isTrue;
    }
    /**
     * @description Checks if the value is an empty value (always false)
     */
    get isTrue() {
        return this.valueOf();
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return this.valueOf() === (other instanceof Boolean
            ? other.valueOf()
            : other);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {
            outer: [this.toU8a()]
        };
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex() {
        return u8aToHex(this.toU8a());
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return this.toJSON();
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        return this.valueOf();
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive() {
        return this.toJSON();
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'bool';
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.toJSON().toString();
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare) {
        return new Uint8Array([this.valueOf() ? 1 : 0]);
    }
}
