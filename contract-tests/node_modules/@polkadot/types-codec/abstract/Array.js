import { compactToU8a, u8aConcatStrict, u8aToHex } from '@polkadot/util';
import { compareArray } from '../utils/compareArray.js';
/**
 * @name AbstractArray
 * @description
 * This manages codec arrays. It is an extension to Array, providing
 * specific encoding/decoding on top of the base type.
 * @noInheritDoc
 */
export class AbstractArray extends Array {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    /**
     * @description This ensures that operators such as clice, filter, map, etc. return
     * new Array instances (without this we need to apply overrides)
     */
    static get [Symbol.species]() {
        return Array;
    }
    constructor(registry, length) {
        super(length);
        this.registry = registry;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        // We need to loop through all entries since they may have a variable length themselves,
        // e.g. when a Vec or Compact is contained withing, it has a variable length based on data
        const count = this.length;
        let total = compactToU8a(count).length;
        for (let i = 0; i < count; i++) {
            total += this[i].encodedLength;
        }
        return total;
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
        return compareArray(this, other);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {
            inner: this.inspectInner(),
            outer: [compactToU8a(this.length)]
        };
    }
    /**
     * @internal
     * @description Internal per-item inspection of internal values
     */
    inspectInner() {
        const count = this.length;
        const inner = new Array(count);
        for (let i = 0; i < count; i++) {
            inner[i] = this[i].inspect();
        }
        return inner;
    }
    /**
     * @description Converts the Object to an standard JavaScript Array
     */
    toArray() {
        return Array.from(this);
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
    toHuman(isExtended, disableAscii) {
        const count = this.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            result[i] = this[i] && this[i].toHuman(isExtended, disableAscii);
        }
        return result;
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        const count = this.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            // We actually log inside the U8a decoding and use JSON.stringify(...), which
            // means that the Vec may be partially populated (same applies to toHuman, same check)
            result[i] = this[i] && this[i].toJSON();
        }
        return result;
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        const count = this.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            result[i] = this[i] && this[i].toPrimitive(disableAscii);
        }
        return result;
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        const count = this.length;
        const result = new Array(count);
        for (let i = 0; i < count; i++) {
            result[i] = this[i].toString();
        }
        return `[${result.join(', ')}]`;
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        const encoded = this.toU8aInner();
        return isBare
            ? u8aConcatStrict(encoded)
            : u8aConcatStrict([compactToU8a(this.length), ...encoded]);
    }
    /**
     * @internal
     * @description Internal per-item SCALE encoding of contained values
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8aInner(isBare) {
        const count = this.length;
        const encoded = new Array(count);
        for (let i = 0; i < count; i++) {
            encoded[i] = this[i].toU8a(isBare);
        }
        return encoded;
    }
}
