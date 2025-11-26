"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Option = void 0;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../utils/index.js");
const Null_js_1 = require("./Null.js");
class None extends Null_js_1.Null {
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'None';
    }
}
/** @internal */
function decodeOption(registry, Type, value) {
    if (value instanceof Type) {
        // don't re-create, use as it (which also caters for derived types)
        return value;
    }
    else if (value instanceof Option) {
        if (value.value instanceof Type) {
            // same instance, return it
            return value.value;
        }
        else if (value.isNone) {
            // internal is None, we are also none
            return new None(registry);
        }
        // convert the actual value into known
        return new Type(registry, value.value);
    }
    else if ((0, util_1.isNull)(value) || (0, util_1.isUndefined)(value) || value === '0x' || value instanceof None) {
        // anything empty we pass as-is
        return new None(registry);
    }
    else if ((0, util_1.isU8a)(value)) {
        // the isU8a check happens last in the if-tree - since the wrapped value
        // may be an instance of it, so Type and Option checks go in first
        return !value.length || value[0] === 0
            ? new None(registry)
            : new Type(registry, value.subarray(1));
    }
    return new Type(registry, value);
}
/**
 * @name Option
 * @description
 * An Option is an optional field. Basically the first byte indicates that there is
 * is value to follow. If the byte is `1` there is an actual value. So the Option
 * implements that - decodes, checks for optionality and wraps the required structure
 * with a value if/as required/found.
 */
class Option {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #Type;
    #raw;
    constructor(registry, typeName, value, { definition, setDefinition = util_1.identity } = {}) {
        const Type = definition || setDefinition((0, index_js_1.typeToConstructor)(registry, typeName));
        const decoded = (0, util_1.isU8a)(value) && value.length && !(0, util_1.isCodec)(value)
            ? value[0] === 0
                ? new None(registry)
                : new Type(registry, value.subarray(1))
            : decodeOption(registry, Type, value);
        this.registry = registry;
        this.#Type = Type;
        this.#raw = decoded;
        if (decoded?.initialU8aLength) {
            this.initialU8aLength = 1 + decoded.initialU8aLength;
        }
    }
    static with(Type) {
        let definition;
        const setDefinition = (d) => {
            definition = d;
            return d;
        };
        return class extends Option {
            constructor(registry, value) {
                super(registry, Type, value, { definition, setDefinition });
            }
        };
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        // boolean byte (has value, doesn't have) along with wrapped length
        return 1 + this.#raw.encodedLength;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Checks if the Option has no value
     */
    get isEmpty() {
        return this.isNone;
    }
    /**
     * @description Checks if the Option has no value
     */
    get isNone() {
        return this.#raw instanceof None;
    }
    /**
     * @description Checks if the Option has a value
     */
    get isSome() {
        return !this.isNone;
    }
    /**
     * @description The actual value for the Option
     */
    get value() {
        return this.#raw;
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        if (other instanceof Option) {
            return (this.isSome === other.isSome) && this.value.eq(other.value);
        }
        return this.value.eq(other);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        if (this.isNone) {
            return { outer: [new Uint8Array([0])] };
        }
        const { inner, outer = [] } = this.#raw.inspect();
        return {
            inner,
            outer: [new Uint8Array([1]), ...outer]
        };
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex() {
        // This attempts to align with the JSON encoding - actually in this case
        // the isSome value is correct, however the `isNone` may be problematic
        return this.isNone
            ? '0x'
            : (0, util_1.u8aToHex)(this.toU8a().subarray(1));
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended, disableAscii) {
        return this.#raw.toHuman(isExtended, disableAscii);
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        return this.isNone
            ? null
            : this.#raw.toJSON();
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        return this.isNone
            ? null
            : this.#raw.toPrimitive(disableAscii);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(isBare) {
        const wrapped = this.registry.getClassName(this.#Type) || new this.#Type(this.registry).toRawType();
        return isBare
            ? wrapped
            : `Option<${wrapped}>`;
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.#raw.toString();
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        if (isBare) {
            return this.#raw.toU8a(true);
        }
        const u8a = new Uint8Array(this.encodedLength);
        if (this.isSome) {
            u8a.set([1]);
            u8a.set(this.#raw.toU8a(), 1);
        }
        return u8a;
    }
    /**
     * @description Returns the value that the Option represents (if available), throws if null
     */
    unwrap() {
        if (this.isNone) {
            throw new Error('Option: unwrapping a None value');
        }
        return this.#raw;
    }
    /**
     * @description Returns the value that the Option represents (if available) or defaultValue if none
     * @param defaultValue The value to return if the option isNone
     */
    unwrapOr(defaultValue) {
        return this.isSome
            ? this.unwrap()
            : defaultValue;
    }
    /**
     * @description Returns the value that the Option represents (if available) or defaultValue if none
     * @param defaultValue The value to return if the option isNone
     */
    unwrapOrDefault() {
        return this.isSome
            ? this.unwrap()
            : new this.#Type(this.registry);
    }
}
exports.Option = Option;
