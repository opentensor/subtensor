"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericAccountId33 = exports.GenericAccountId = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
/** @internal */
function decodeAccountId(value) {
    if ((0, util_1.isU8a)(value) || Array.isArray(value)) {
        return (0, util_1.u8aToU8a)(value);
    }
    else if (!value) {
        return new Uint8Array();
    }
    else if ((0, util_1.isHex)(value)) {
        return (0, util_1.hexToU8a)(value);
    }
    else if ((0, util_1.isString)(value)) {
        return (0, util_crypto_1.decodeAddress)(value.toString());
    }
    throw new Error(`Unknown type passed to AccountId constructor, found typeof ${typeof value}`);
}
class BaseAccountId extends types_codec_1.U8aFixed {
    constructor(registry, allowedBits = 256 | 264, value) {
        const decoded = decodeAccountId(value);
        const decodedBits = decoded.length * 8;
        // Part of stream containing >= 32 bytes or a all empty (defaults)
        if (decodedBits < allowedBits && decoded.some((b) => b)) {
            throw new Error(`Invalid AccountId provided, expected ${allowedBits >> 3} bytes, found ${decoded.length}`);
        }
        super(registry, decoded, allowedBits);
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return super.eq(decodeAccountId(other));
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
        return (0, util_crypto_1.encodeAddress)(this, this.registry.chainSS58);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'AccountId';
    }
}
/**
 * @name GenericAccountId
 * @description
 * A wrapper around an AccountId/PublicKey representation. Since we are dealing with
 * underlying PublicKeys (32 bytes in length), we extend from U8aFixed which is
 * just a Uint8Array wrapper with a fixed length.
 * If constructed with an empty value ([], "", undefined) it will result in
 * the zero account 0x000...000.
 */
class GenericAccountId extends BaseAccountId {
    constructor(registry, value) {
        super(registry, 256, value);
    }
}
exports.GenericAccountId = GenericAccountId;
class GenericAccountId33 extends BaseAccountId {
    constructor(registry, value) {
        super(registry, 264, value);
    }
}
exports.GenericAccountId33 = GenericAccountId33;
