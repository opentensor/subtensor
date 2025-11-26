"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Text = void 0;
const util_1 = require("@polkadot/util");
const Raw_js_1 = require("./Raw.js");
const MAX_LENGTH = 128 * 1024;
/** @internal */
function decodeText(value) {
    if ((0, util_1.isU8a)(value)) {
        if (!value.length) {
            return ['', 0];
        }
        // for Raw, the internal buffer does not have an internal length
        // (the same applies in e.g. Bytes, where length is added at encoding-time)
        if (value instanceof Raw_js_1.Raw) {
            return [(0, util_1.u8aToString)(value), 0];
        }
        const [offset, length] = (0, util_1.compactFromU8aLim)(value);
        const total = offset + length;
        if (length > MAX_LENGTH) {
            throw new Error(`Text: length ${length.toString()} exceeds ${MAX_LENGTH}`);
        }
        else if (total > value.length) {
            throw new Error(`Text: required length less than remainder, expected at least ${total}, found ${value.length}`);
        }
        return [(0, util_1.u8aToString)(value.subarray(offset, total)), total];
    }
    else if ((0, util_1.isHex)(value)) {
        return [(0, util_1.u8aToString)((0, util_1.hexToU8a)(value)), 0];
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
class Text extends String {
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
        return (0, util_1.isString)(other)
            ? this.toString() === other.toString()
            : false;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        const value = (0, util_1.stringToU8a)(super.toString());
        return {
            outer: value.length
                ? [(0, util_1.compactToU8a)(value.length), value]
                : [(0, util_1.compactToU8a)(value.length)]
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
        return (0, util_1.u8aToHex)(this.toU8a(true));
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
        const encoded = (0, util_1.stringToU8a)(super.toString());
        return isBare
            ? encoded
            : (0, util_1.compactAddLength)(encoded);
    }
}
exports.Text = Text;
