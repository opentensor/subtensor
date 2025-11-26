"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericAccountIndex = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const PREFIX_1BYTE = 0xef;
const PREFIX_2BYTE = 0xfc;
const PREFIX_4BYTE = 0xfd;
const PREFIX_8BYTE = 0xfe;
const MAX_1BYTE = new util_1.BN(PREFIX_1BYTE);
const MAX_2BYTE = new util_1.BN(1).shln(16);
const MAX_4BYTE = new util_1.BN(1).shln(32);
/** @internal */
function decodeAccountIndex(value) {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (value instanceof GenericAccountIndex) {
        // `value.toBn()` on AccountIndex returns a pure BN (i.e. not an
        // AccountIndex), which has the initial `toString()` implementation.
        return value.toBn();
    }
    else if ((0, util_1.isBn)(value) || (0, util_1.isNumber)(value) || (0, util_1.isHex)(value) || (0, util_1.isU8a)(value) || (0, util_1.isBigInt)(value)) {
        return value;
    }
    return decodeAccountIndex((0, util_crypto_1.decodeAddress)(value));
}
/**
 * @name GenericAccountIndex
 * @description
 * A wrapper around an AccountIndex, which is a shortened, variable-length encoding
 * for an Account. We extends from [[U32]] to provide the number-like properties.
 */
class GenericAccountIndex extends types_codec_1.u32 {
    constructor(registry, value = new util_1.BN(0)) {
        super(registry, decodeAccountIndex(value));
    }
    static calcLength(_value) {
        const value = (0, util_1.bnToBn)(_value);
        if (value.lte(MAX_1BYTE)) {
            return 1;
        }
        else if (value.lt(MAX_2BYTE)) {
            return 2;
        }
        else if (value.lt(MAX_4BYTE)) {
            return 4;
        }
        return 8;
    }
    static readLength(input) {
        const first = input[0];
        if (first === PREFIX_2BYTE) {
            return [1, 2];
        }
        else if (first === PREFIX_4BYTE) {
            return [1, 4];
        }
        else if (first === PREFIX_8BYTE) {
            return [1, 8];
        }
        return [0, 1];
    }
    static writeLength(input) {
        switch (input.length) {
            case 2: return new Uint8Array([PREFIX_2BYTE]);
            case 4: return new Uint8Array([PREFIX_4BYTE]);
            case 8: return new Uint8Array([PREFIX_8BYTE]);
            default: return new Uint8Array([]);
        }
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        // shortcut for BN or Number, don't create an object
        if ((0, util_1.isBn)(other) || (0, util_1.isNumber)(other)) {
            return super.eq(other);
        }
        // convert and compare
        return super.eq(this.registry.createTypeUnsafe('AccountIndex', [other]));
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
     * @description Returns the string representation of the value
     */
    toString() {
        const length = GenericAccountIndex.calcLength(this);
        return (0, util_crypto_1.encodeAddress)(this.toU8a().subarray(0, length), this.registry.chainSS58);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'AccountIndex';
    }
}
exports.GenericAccountIndex = GenericAccountIndex;
