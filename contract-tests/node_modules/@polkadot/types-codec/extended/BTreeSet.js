import { compactFromU8aLim, compactToU8a, isHex, isU8a, logger, stringify, u8aConcatStrict, u8aToHex, u8aToU8a } from '@polkadot/util';
import { compareSet, decodeU8aVec, sortSet, typeToConstructor } from '../utils/index.js';
const l = logger('BTreeSet');
/** @internal */
function decodeSetFromU8a(registry, ValClass, u8a) {
    const output = new Set();
    const [offset, count] = compactFromU8aLim(u8a);
    const result = new Array(count);
    const [decodedLength] = decodeU8aVec(registry, result, u8a, offset, ValClass);
    for (let i = 0; i < count; i++) {
        output.add(result[i]);
    }
    return [ValClass, output, decodedLength];
}
/** @internal */
function decodeSetFromSet(registry, ValClass, value) {
    const output = new Set();
    value.forEach((val) => {
        try {
            output.add((val instanceof ValClass) ? val : new ValClass(registry, val));
        }
        catch (error) {
            l.error('Failed to decode key or value:', error.message);
            throw error;
        }
    });
    return [ValClass, output, 0];
}
/**
 * Decode input to pass into constructor.
 *
 * @param ValClass - Type of the map value
 * @param value - Value to decode, one of:
 * - null
 * - undefined
 * - hex
 * - Uint8Array
 * - Set<any>, where both key and value types are either
 *   constructors or decodeable values for their types.
 * @param jsonSet
 * @internal
 */
function decodeSet(registry, valType, value) {
    const ValClass = typeToConstructor(registry, valType);
    if (!value) {
        return [ValClass, new Set(), 0];
    }
    else if (isU8a(value) || isHex(value)) {
        return decodeSetFromU8a(registry, ValClass, u8aToU8a(value));
    }
    else if (Array.isArray(value) || value instanceof Set) {
        return decodeSetFromSet(registry, ValClass, value);
    }
    throw new Error('BTreeSet: cannot decode type');
}
export class BTreeSet extends Set {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #ValClass;
    constructor(registry, valType, rawValue) {
        const [ValClass, values, decodedLength] = decodeSet(registry, valType, rawValue);
        super(sortSet(values));
        this.registry = registry;
        this.initialU8aLength = decodedLength;
        this.#ValClass = ValClass;
    }
    static with(valType) {
        return class extends BTreeSet {
            constructor(registry, value) {
                super(registry, valType, value);
            }
        };
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        let len = compactToU8a(this.size).length;
        for (const v of this.values()) {
            len += v.encodedLength;
        }
        return len;
    }
    /**
     * @description Returns a hash of the value
     */
    get hash() {
        return this.registry.hash(this.toU8a());
    }
    /**
     * @description Checks if the value is an empty value
     */
    get isEmpty() {
        return this.size === 0;
    }
    /**
     * @description The actual set values as a string[]
     */
    get strings() {
        return [...super.values()].map((v) => v.toString());
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return compareSet(this, other);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        const inner = [];
        for (const v of this.values()) {
            inner.push(v.inspect());
        }
        return {
            inner,
            outer: [compactToU8a(this.size)]
        };
    }
    /**
     * @description Returns a hex string representation of the value. isLe returns a LE (number-only) representation
     */
    toHex() {
        return u8aToHex(this.toU8a());
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended, disableAscii) {
        const json = [];
        for (const v of this.values()) {
            json.push(v.toHuman(isExtended, disableAscii));
        }
        return json;
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        const json = [];
        for (const v of this.values()) {
            json.push(v.toJSON());
        }
        return json;
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `BTreeSet<${this.registry.getClassName(this.#ValClass) || new this.#ValClass(this.registry).toRawType()}>`;
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        const json = [];
        for (const v of this.values()) {
            json.push(v.toPrimitive(disableAscii));
        }
        return json;
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return stringify(this.toJSON());
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        const encoded = [];
        if (!isBare) {
            encoded.push(compactToU8a(this.size));
        }
        for (const v of this.values()) {
            encoded.push(v.toU8a(isBare));
        }
        return u8aConcatStrict(encoded);
    }
}
