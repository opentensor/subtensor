import { BN, bnToBn, bnToU8a, isBn, isNumber, isString, isU8a, isUndefined, objectProperties, stringify, stringPascalCase, u8aToBn, u8aToHex, u8aToU8a } from '@polkadot/util';
import { compareArray } from '../utils/index.js';
function encodeSet(setValues, values) {
    const encoded = new BN(0);
    for (let i = 0, count = values.length; i < count; i++) {
        encoded.ior(bnToBn(setValues[values[i]] || 0));
    }
    return encoded;
}
/** @internal */
function decodeSetArray(setValues, values) {
    const count = values.length;
    const result = new Array(count);
    for (let i = 0; i < count; i++) {
        const key = values[i];
        if (isUndefined(setValues[key])) {
            throw new Error(`Set: Invalid key '${key}' passed to Set, allowed ${Object.keys(setValues).join(', ')}`);
        }
        result[i] = key;
    }
    return result;
}
/** @internal */
function decodeSetNumber(setValues, _value) {
    const bn = bnToBn(_value);
    const keys = Object.keys(setValues);
    const result = [];
    for (let i = 0, count = keys.length; i < count; i++) {
        const key = keys[i];
        if (bn.and(bnToBn(setValues[key])).eq(bnToBn(setValues[key]))) {
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
    if (isU8a(value)) {
        return value.length === 0
            ? []
            : decodeSetNumber(setValues, u8aToBn(value.subarray(0, byteLength), { isLe: true }));
    }
    else if (isString(value)) {
        return decodeSet(setValues, u8aToU8a(value), byteLength);
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
export class CodecSet extends Set {
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
                    isKeys[i] = `is${stringPascalCase(keys[i])}`;
                }
                objectProperties(this.prototype, isKeys, (_, i, self) => self.strings.includes(keys[i]));
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
        if (this.#allowed && isUndefined(this.#allowed[key])) {
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
            return compareArray(this.strings.sort(), other.sort());
        }
        else if (other instanceof Set) {
            return this.eq([...other.values()]);
        }
        else if (isNumber(other) || isBn(other)) {
            return this.valueEncoded.eq(bnToBn(other));
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
        return stringify({ _set: this.#allowed });
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
        return bnToU8a(this.valueEncoded, {
            bitLength: this.#byteLength * 8,
            isLe: true
        });
    }
}
