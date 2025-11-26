"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Bytes = void 0;
const util_1 = require("@polkadot/util");
const Raw_js_1 = require("../native/Raw.js");
const MAX_LENGTH = 10 * 1024 * 1024;
/** @internal */
function decodeBytesU8a(value) {
    if (!value.length) {
        return [new Uint8Array(), 0];
    }
    // handle all other Uint8Array inputs, these do have a length prefix
    const [offset, length] = (0, util_1.compactFromU8aLim)(value);
    const total = offset + length;
    if (length > MAX_LENGTH) {
        throw new Error(`Bytes length ${length.toString()} exceeds ${MAX_LENGTH}`);
    }
    else if (total > value.length) {
        throw new Error(`Bytes: required length less than remainder, expected at least ${total}, found ${value.length}`);
    }
    return [value.subarray(offset, total), total];
}
/**
 * @name Bytes
 * @description
 * A Bytes wrapper for Vec<u8>. The significant difference between this and a normal Uint8Array
 * is that this version allows for length-encoding. (i.e. it is a variable-item codec, the same
 * as what is found in [[Text]] and [[Vec]])
 */
class Bytes extends Raw_js_1.Raw {
    constructor(registry, value) {
        const [u8a, decodedLength] = (0, util_1.isU8a)(value) && !(value instanceof Raw_js_1.Raw)
            ? decodeBytesU8a(value)
            : Array.isArray(value) || (0, util_1.isString)(value)
                ? [(0, util_1.u8aToU8a)(value), 0]
                : [value, 0];
        super(registry, u8a, decodedLength);
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.length + (0, util_1.compactToU8a)(this.length).length;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(isBare) {
        const clength = (0, util_1.compactToU8a)(this.length);
        return {
            outer: isBare
                ? [super.toU8a()]
                : this.length
                    ? [clength, super.toU8a()]
                    : [clength]
        };
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'Bytes';
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        return isBare
            ? super.toU8a(isBare)
            : (0, util_1.compactAddLength)(this);
    }
}
exports.Bytes = Bytes;
