"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CodecSet = void 0;
const util_1 = require("@polkadot/util");
const index_js_1 = require("../utils/index.js");
function encodeSet(setValues, values) {
    const encoded = new util_1.BN(0);
    for (let i = 0, count = values.length; i < count; i++) {
        encoded.ior((0, util_1.bnToBn)(setValues[values[i]] || 0));
    }
    return encoded;
}
/** @internal */
function decodeSetArray(setValues, values) {
    const count = values.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        const key = values[i];
        if ((0, util_1.isUndefined)(setValues[key])) {
            throw new Error(`Set: Invalid key '${key}' passed to Set, allowed ${Object.keys(setValues).join(', ')}`);
        }
        result[i] = key;
    }
    return result;
}
/** @internal */
function decodeSetNumber(setValues, _value) {
    const bn = (0, util_1.bnToBn)(_value);
    const keys = Object.keys(setValues);
    const result = [];
    for (let i = 0, count = keys.length; i < count; i++) {
        const key = keys[i];
        if (bn.and((0, util_1.bnToBn)(setValues[key])).eq((0, util_1.bnToBn)(setValues[key]))) {
            result.push(key);
        }
    }
    const computed = encodeSet(setValues, result);
    if (!bn.eq(computed)) {
        throw new Error(`Set: Mismatch decoding '${bn.toString()}', computed as '${computed.toString()}' with ${result.join(', ')}`);
    }
    return result;
}
/** @internal */
function decodeSet(setValues, value = 0, bitLength) {
    if (bitLength % 8 !== 0) {
        throw new Error(`Expected valid bitLength, power of 8, found ${bitLength}`);
    }
    const byteLength = bitLength / 8;
    if ((0, util_1.isU8a)(value)) {
        return value.length === 0
            ? []
            : decodeSetNumber(setValues, (0, util_1.u8aToBn)(value.subarray(0, byteLength), { isLe: true }));
    }
    else if ((0, util_1.isString)(value)) {
        return decodeSet(setValues, (0, util_1.u8aToU8a)(value), byteLength);
    }
    else if (value instanceof Set || Array.isArray(value)) {
        const input = Array.isArray(value)
            ? value
            : [...value.values()];
        return decodeSetArray(setValues, input);
    }
    return decodeSetNumber(setValues, value);
}
/**
 * @name Set
 * @description
 * An Set is an array of string values, represented an an encoded type by
 * a bitwise representation of the values.
 */
class CodecSet extends Set {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #allowed;
    #byteLength;
    constructor(registry, setValues, value, bitLength = 8) {
        super(decodeSet(setValues, value, bitLength));
        this.registry = registry;
        this.#allowed = setValues;
        this.#byteLength = bitLength / 8;
    }
    static with(values, bitLength) {
        return class extends CodecSet {
            static {
                const keys = Object.keys(values);
                const count = keys.length;
                const isKeys = new Array(count);
                for (let i = 0; i < count; i++) {
                    isKeys[i] = `is${(0, util_1.stringPascalCase)(keys[i])}`;
                }
                (0, util_1.objectProperties)(this.prototype, isKeys, (_, i, self) => self.strings.includes(keys[i]));
            }
            constructor(registry, value) {
                super(registry, values, value, bitLength);
            }
        };
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.#byteLength;
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description true is the Set contains no values
     */
    get isEmpty() {
        return this.size === 0;
    }
    /**
     * @description The actual set values as a string[]
     */
    get strings() {
        return [...super.values()];
    }
    /**
     * @description The encoded value for the set members
     */
    get valueEncoded() {
        return encodeSet(this.#allowed, this.strings);
    }
    /**
     * @description adds a value to the Set (extended to allow for validity checking)
     */
    add = (key) => {
        // ^^^ add = () property done to assign this instance's this, otherwise Set.add creates "some" chaos
        // we have the isUndefined(this._setValues) in here as well, add is used internally
        // in the Set constructor (so it is undefined at this point, and should allow)
        if (this.#allowed && (0, util_1.isUndefined)(this.#allowed[key])) {
            throw new Error(`Set: Invalid key '${key}' on add`);
        }
        super.add(key);
        return this;
    };
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        if (Array.isArray(other)) {
            // we don't actually care about the order, sort the values
            return (0, index_js_1.compareArray)(this.strings.sort(), other.sort());
        }
        else if (other instanceof Set) {
            return this.eq([...other.values()]);
        }
        else if ((0, util_1.isNumber)(other) || (0, util_1.isBn)(other)) {
            return this.valueEncoded.eq((0, util_1.bnToBn)(other));
        }
        return false;
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
        return (0, util_1.u8aToHex)(this.toU8a());
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
        return this.strings;
    }
    /**
     * @description The encoded value for the set members
     */
    toNumber() {
        return this.valueEncoded.toNumber();
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
        return (0, util_1.stringify)({ _set: this.#allowed });
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return `[${this.strings.join(', ')}]`;
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare) {
        return (0, util_1.bnToU8a)(this.valueEncoded, {
            bitLength: this.#byteLength * 8,
            isLe: true
        });
    }
}
exports.CodecSet = CodecSet;
