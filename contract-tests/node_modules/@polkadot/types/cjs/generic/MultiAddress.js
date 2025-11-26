"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericMultiAddress = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const util_crypto_1 = require("@polkadot/util-crypto");
const AccountId_js_1 = require("./AccountId.js");
const AccountIndex_js_1 = require("./AccountIndex.js");
function decodeU8a(registry, u8a) {
    if ([0, 32].includes(u8a.length)) {
        return { Id: u8a };
    }
    else if (u8a.length === 20) {
        return { Address20: u8a };
    }
    else if (u8a.length <= 8) {
        return { Index: registry.createTypeUnsafe('AccountIndex', [u8a]).toNumber() };
    }
    return u8a;
}
function decodeMultiAny(registry, value) {
    if (value instanceof AccountId_js_1.GenericAccountId) {
        return { Id: value };
    }
    else if ((0, util_1.isU8a)(value)) {
        // NOTE This is after the AccountId check (which is U8a)
        return decodeU8a(registry, value);
    }
    else if (value instanceof GenericMultiAddress) {
        return value;
    }
    else if (value instanceof AccountIndex_js_1.GenericAccountIndex || (0, util_1.isBn)(value) || (0, util_1.isNumber)(value)) {
        return { Index: (0, util_1.isNumber)(value) ? value : value.toNumber() };
    }
    else if ((0, util_1.isString)(value)) {
        return decodeU8a(registry, (0, util_crypto_1.decodeAddress)(value.toString()));
    }
    return value;
}
class GenericMultiAddress extends types_codec_1.Enum {
    constructor(registry, value) {
        super(registry, {
            Id: 'AccountId',
            Index: 'Compact<AccountIndex>',
            Raw: 'Bytes',
            // eslint-disable-next-line sort-keys
            Address32: 'H256',
            // eslint-disable-next-line sort-keys
            Address20: 'H160'
        }, decodeMultiAny(registry, value));
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        const { inner, outer = [] } = this.inner.inspect();
        return {
            inner,
            outer: [new Uint8Array([this.index]), ...outer]
        };
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.value.toString();
    }
}
exports.GenericMultiAddress = GenericMultiAddress;
