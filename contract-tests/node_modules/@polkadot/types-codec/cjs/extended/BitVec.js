"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BitVec = void 0;
const util_1 = require("@polkadot/util");
const Raw_js_1 = require("../native/Raw.js");
/** @internal */
function decodeBitVecU8a(value) {
    if (!value?.length) {
        return [0, new Uint8Array()];
    }
    // handle all other Uint8Array inputs, these do have a length prefix which is the number of bits encoded
    const [offset, length] = (0, util_1.compactFromU8aLim)(value);
    const total = offset + Math.ceil(length / 8);
    if (total > value.length) {
        throw new Error(`BitVec: required length less than remainder, expected at least ${total}, found ${value.length}`);
    }
    return [length, value.subarray(offset, total)];
}
/** @internal */
function decodeBitVec(value) {
    if (Array.isArray(value) || (0, util_1.isString)(value)) {
        const u8a = (0, util_1.u8aToU8a)(value);
        return [u8a.length * 8, u8a];
    }
    return decodeBitVecU8a(value);
}
/**
 * @name BitVec
 * @description
 * A BitVec that represents an array of bits. The bits are however stored encoded. The difference between this
 * and a normal Bytes would be that the length prefix indicates the number of bits encoded, not the bytes
 */
class BitVec extends Raw_js_1.Raw {
    #decodedLength;
    #isMsb;
    // In lieu of having the Msb/Lsb identifiers passed through, we default to assuming
    // we are dealing with Lsb, which is the default (as of writing) BitVec format used
    // in the Polkadot code (this only affects the toHuman displays)
    constructor(registry, value, isMsb = false) {
        const [decodedLength, u8a] = decodeBitVec(value);
        super(registry, u8a);
        this.#decodedLength = decodedLength;
        this.#isMsb = isMsb;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.length + (0, util_1.compactToU8a)(this.#decodedLength).length;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return {
            outer: [(0, util_1.compactToU8a)(this.#decodedLength), super.toU8a()]
        };
    }
    /**
     * @description Creates a boolean array of the bit values
     */
    toBoolArray() {
        const map = [...this.toU8a(true)].map((v) => [
            !!(v & 0b1000_0000),
            !!(v & 0b0100_0000),
            !!(v & 0b0010_0000),
            !!(v & 0b0001_0000),
            !!(v & 0b0000_1000),
            !!(v & 0b0000_0100),
            !!(v & 0b0000_0010),
            !!(v & 0b0000_0001)
        ]);
        const count = map.length;
        const result = new Array(8 * count);
        for (let i = 0; i < count; i++) {
            const off = i * 8;
            const v = map[i];
            for (let j = 0; j < 8; j++) {
                result[off + j] = this.#isMsb
                    ? v[j]
                    : v[7 - j];
            }
        }
        return result;
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman() {
        return `0b${[...this.toU8a(true)]
            .map((d) => `00000000${d.toString(2)}`.slice(-8))
            .map((s) => this.#isMsb ? s : s.split('').reverse().join(''))
            .join('_')}`;
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'BitVec';
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        const bitVec = super.toU8a(isBare);
        return isBare
            ? bitVec
            : (0, util_1.u8aConcatStrict)([(0, util_1.compactToU8a)(this.#decodedLength), bitVec]);
    }
}
exports.BitVec = BitVec;
