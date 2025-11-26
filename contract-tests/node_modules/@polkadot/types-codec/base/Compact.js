import { compactFromU8a, compactFromU8aLim, compactToU8a, identity, isU8a } from '@polkadot/util';
import { typeToConstructor } from '../utils/index.js';
function decodeCompact(registry, Type, value) {
    if (isU8a(value)) {
        const [decodedLength, bn] = (value[0] & 0b11) < 0b11
            ? compactFromU8aLim(value)
            : compactFromU8a(value);
        return [new Type(registry, bn), decodedLength];
    }
    else if (value instanceof Compact) {
        const raw = value.unwrap();
        return raw instanceof Type
            ? [raw, 0]
            : [new Type(registry, raw), 0];
    }
    else if (value instanceof Type) {
        return [value, 0];
    }
    return [new Type(registry, value), 0];
}
/**
 * @name Compact
 * @description
 * A compact length-encoding codec wrapper. It performs the same function as Length, however
 * differs in that it uses a variable number of bytes to do the actual encoding. This is mostly
 * used by other types to add length-prefixed encoding, or in the case of wrapped types, taking
 * a number and making the compact representation thereof
 */
export class Compact {
    registry;
    createdAtHash;
    initialU8aLength;
    isStorageFallback;
    #Type;
    #raw;
    constructor(registry, Type, value = 0, { definition, setDefinition = identity } = {}) {
        this.registry = registry;
        this.#Type = definition || setDefinition(typeToConstructor(registry, Type));
        const [raw, decodedLength] = decodeCompact(registry, this.#Type, value);
        this.initialU8aLength = decodedLength;
        this.#raw = raw;
    }
    static with(Type) {
        let definition;
        // eslint-disable-next-line no-return-assign
        const setDefinition = (d) => (definition = d);
        return class extends Compact {
            constructor(registry, value) {
                super(registry, Type, value, { definition, setDefinition });
            }
        };
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
        return this.#raw.isEmpty;
    }
    /**
     * @description Returns the number of bits in the value
     */
    bitLength() {
        return this.#raw.bitLength();
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return this.#raw.eq(other instanceof Compact
            ? other.#raw
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
     * @description Returns a BigInt representation of the number
     */
    toBigInt() {
        return this.#raw.toBigInt();
    }
    /**
     * @description Returns the BN representation of the number
     */
    toBn() {
        return this.#raw.toBn();
    }
    /**
     * @description Returns a hex string representation of the value. isLe returns a LE (number-only) representation
     */
    toHex(isLe) {
        return this.#raw.toHex(isLe);
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
        return this.#raw.toJSON();
    }
    /**
     * @description Returns the number representation for the value
     */
    toNumber() {
        return this.#raw.toNumber();
    }
    /**
     * @description Converts the value in a best-fit primitive form
     */
    toPrimitive(disableAscii) {
        return this.#raw.toPrimitive(disableAscii);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return `Compact<${this.registry.getClassName(this.#Type) || this.#raw.toRawType()}>`;
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.#raw.toString();
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     */
    toU8a(_isBare) {
        return compactToU8a(this.#raw.toBn());
    }
    /**
     * @description Returns the embedded [[UInt]] or [[Moment]] value
     */
    unwrap() {
        return this.#raw;
    }
}
