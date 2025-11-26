import { compactAddLength, compactFromU8aLim, compactToU8a, hexToU8a, isHex, isString, isU8a, stringToU8a, u8aToHex, u8aToString } from '@polkadot/util';
import { Raw } from './Raw.js';
const MAX_LENGTH = 128 * 1024;
/** @internal */
function decodeText(value) {
    if (isU8a(value)) {
        if (!value.length) {
            return ['', 0];
        }
        // for Raw, the internal buffer does not have an internal length
        // (the same applies in e.g. Bytes, where length is added at encoding-time)
        if (value instanceof Raw) {
            return [u8aToString(value), 0];
        }
        const [offset, length] = compactFromU8aLim(value);
        const total = offset + length;
        if (length > MAX_LENGTH) {
            throw new Error(`Text: length ${length.toString()} exceeds ${MAX_LENGTH}`);
        }
        else if (total > value.length) {
            throw new Error(`Text: required length less than remainder, expected at least ${total}, found ${value.length}`);
        }
        return [u8aToString(value.subarray(offset, total)), total];
    }
    else if (isHex(value)) {
        return [u8aToString(hexToU8a(value)), 0];
    }
    return [value ? value.toString() : '', 0];
}
/**
 * @name Text
 * @description
 * This is a string wrapper, along with the length. It is used both for strings as well
 * as items such as documentation. It simply extends the standard JS `String` built-in
 * object, inheriting all methods exposed from `String`.
 * @noInheritDoc
 */
export class Text extends String {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #override = null;
    constructor(registry, value) {
        const [str, decodedLength] = decodeText(value);
        super(str);
        this.registry = registry;
        this.initialU8aLength = decodedLength;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.toU8a().length;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Checks if the value is an empty value
     */
    get isEmpty() {
        return this.length === 0;
    }
    /**
     * @description The length of the value
     */
    get length() {
        // only included here since we ignore inherited docs
        return super.length;
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return isString(other)
            ? this.toString() === other.toString()
            : false;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        const value = stringToU8a(super.toString());
        return {
            outer: value.length
                ? [compactToU8a(value.length), value]
                : [compactToU8a(value.length)]
        };
    }
    /**
     * @description Set an override value for this
     */
    setOverride(override) {
        this.#override = override;
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex() {
        // like with Vec<u8>, when we are encoding to hex, we don't actually add
        // the length prefix (it is already implied by the actual string length)
        return u8aToHex(this.toU8a(true));
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
        return this.toString();
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
        return 'Text';
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.#override || super.toString();
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        // NOTE Here we use the super toString (we are not taking overrides into account,
        // rather encoding the original value the string was constructed with)
        const encoded = stringToU8a(super.toString());
        return isBare
            ? encoded
            : compactAddLength(encoded);
    }
}
