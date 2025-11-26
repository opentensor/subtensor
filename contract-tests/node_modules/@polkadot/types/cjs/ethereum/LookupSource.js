"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericEthereumLookupSource = exports.ACCOUNT_ID_PREFIX = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const AccountIndex_js_1 = require("../generic/AccountIndex.js");
const AccountId_js_1 = require("./AccountId.js");
exports.ACCOUNT_ID_PREFIX = new Uint8Array([0xff]);
/** @internal */
function decodeString(registry, value) {
    const decoded = (0, util_crypto_1.decodeAddress)(value);
    return decoded.length === 20
        ? registry.createTypeUnsafe('EthereumAccountId', [decoded])
        : registry.createTypeUnsafe('AccountIndex', [(0, util_1.u8aToBn)(decoded)]);
}
/** @internal */
function decodeU8a(registry, value) {
    // This allows us to instantiate an address with a raw publicKey. Do this first before
    // we checking the first byte, otherwise we may split an already-existent valid address
    if (value.length === 20) {
        return registry.createTypeUnsafe('EthereumAccountId', [value]);
    }
    else if (value[0] === 0xff) {
        return registry.createTypeUnsafe('EthereumAccountId', [value.subarray(1)]);
    }
    const [offset, length] = AccountIndex_js_1.GenericAccountIndex.readLength(value);
    return registry.createTypeUnsafe('AccountIndex', [(0, util_1.u8aToBn)(value.subarray(offset, offset + length))]);
}
function decodeAddressOrIndex(registry, value) {
    return value instanceof GenericEthereumLookupSource
        ? value.inner
        : value instanceof AccountId_js_1.GenericEthereumAccountId || value instanceof AccountIndex_js_1.GenericAccountIndex
            ? value
            : (0, util_1.isU8a)(value) || Array.isArray(value) || (0, util_1.isHex)(value)
                ? decodeU8a(registry, (0, util_1.u8aToU8a)(value))
                : (0, util_1.isBn)(value) || (0, util_1.isNumber)(value) || (0, util_1.isBigInt)(value)
                    ? registry.createTypeUnsafe('AccountIndex', [value])
                    : decodeString(registry, value);
}
/**
 * @name GenericEthereumLookupSource
 * @description
 * A wrapper around an EthereumAccountId and/or AccountIndex that is encoded with a prefix.
 * Since we are dealing with underlying publicKeys (or shorter encoded addresses),
 * we extend from Base with an AccountId/AccountIndex wrapper. Basically the Address
 * is encoded as `[ <prefix-byte>, ...publicKey/...bytes ]` as per spec
 */
class GenericEthereumLookupSource extends types_codec_1.AbstractBase {
    constructor(registry, value = new Uint8Array()) {
        super(registry, decodeAddressOrIndex(registry, value));
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        const rawLength = this._rawLength;
        return rawLength + (
        // for 1 byte AccountIndexes, we are not adding a specific prefix
        rawLength > 1
            ? 1
            : 0);
    }
    /**
     * @description The length of the raw value, either AccountIndex or AccountId
     */
    get _rawLength() {
        return this.inner instanceof AccountIndex_js_1.GenericAccountIndex
            ? AccountIndex_js_1.GenericAccountIndex.calcLength(this.inner)
            : this.inner.encodedLength;
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex() {
        return (0, util_1.u8aToHex)(this.toU8a());
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'Address';
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        const encoded = this.inner.toU8a().subarray(0, this._rawLength);
        return isBare
            ? encoded
            : (0, util_1.u8aConcat)(this.inner instanceof AccountIndex_js_1.GenericAccountIndex
                ? AccountIndex_js_1.GenericAccountIndex.writeLength(encoded)
                : exports.ACCOUNT_ID_PREFIX, encoded);
    }
}
exports.GenericEthereumLookupSource = GenericEthereumLookupSource;
