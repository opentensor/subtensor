"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericEthereumAccountId = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
/** @internal */
function decodeAccountId(value) {
    if ((0, util_1.isU8a)(value) || Array.isArray(value)) {
        return (0, util_1.u8aToU8a)(value);
    }
    else if ((0, util_1.isHex)(value) || (0, util_crypto_1.isEthereumAddress)(value.toString())) {
        return (0, util_1.hexToU8a)(value.toString());
    }
    else if ((0, util_1.isString)(value)) {
        return (0, util_1.u8aToU8a)(value);
    }
    return value;
}
/**
 * @name GenericEthereumAccountId
 * @description
 * A wrapper around an Ethereum-compatible AccountId. Since we are dealing with
 * underlying addresses (20 bytes in length), we extend from U8aFixed which is
 * just a Uint8Array wrapper with a fixed length.
 */
class GenericEthereumAccountId extends types_codec_1.U8aFixed {
    constructor(registry, value = new Uint8Array()) {
        super(registry, decodeAccountId(value), 160);
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return !!other && super.eq(decodeAccountId(other));
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
        return (0, util_crypto_1.ethereumEncode)(this);
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'AccountId';
    }
}
exports.GenericEthereumAccountId = GenericEthereumAccountId;
