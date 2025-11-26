import { AbstractBase } from '@polkadot/types-codec';
import { hexToU8a, isHex, u8aToHex } from '@polkadot/util';
import { DEFAULT_PREAMBLE, LATEST_EXTRINSIC_VERSION } from './constants.js';
const VERSIONS = [
    'ExtrinsicPayloadUnknown', // v0 is unknown
    'ExtrinsicPayloadUnknown',
    'ExtrinsicPayloadUnknown',
    'ExtrinsicPayloadUnknown',
    'ExtrinsicPayloadV4',
    'ExtrinsicPayloadV5'
];
const PREAMBLES = {
    bare: 'ExtrinsicPayloadV5',
    // Not supported yet
    general: 'ExtrinsicPayloadV5'
};
/**
 * HACK: In order to change the assetId from `number | object` to HexString (While maintaining the true type ie Option<TAssetConversion>),
 * to allow for easier generalization of the SignerPayloadJSON interface the below check is necessary. The ExtrinsicPayloadV4 class does not like
 * a value passed in as an Option, and can't decode it properly. Therefore, we ensure to convert the following below, and then pass the option as a unwrapped
 * JSON value.
 *
 * ref: https://github.com/polkadot-js/api/pull/5968
 * ref: https://github.com/polkadot-js/api/pull/5967
 */
export function decodeAssetId(registry, payload) {
    const maybeAsset = payload?.assetId;
    if (maybeAsset && isHex(maybeAsset)) {
        const assetId = registry.createType('TAssetConversion', hexToU8a(maybeAsset));
        // we only want to adjust the payload if the hex passed has the option
        if (maybeAsset === '0x00' || maybeAsset === '0x01' + assetId.toHex().slice(2)) {
            return {
                ...payload,
                assetId: assetId.toJSON()
            };
        }
    }
    return payload;
}
/** @internal */
function decodeExtrinsicPayload(registry, value, version = LATEST_EXTRINSIC_VERSION, preamble = DEFAULT_PREAMBLE) {
    if (value instanceof GenericExtrinsicPayload) {
        return value.unwrap();
    }
    const extVersion = version === 5 ? PREAMBLES[preamble] : VERSIONS[version] || VERSIONS[0];
    const payload = decodeAssetId(registry, value);
    return registry.createTypeUnsafe(extVersion, [payload, { version }]);
}
/**
 * @name GenericExtrinsicPayload
 * @description
 * A signing payload for an [[Extrinsic]]. For the final encoding, it is variable length based
 * on the contents included
 */
export class GenericExtrinsicPayload extends AbstractBase {
    constructor(registry, value, { preamble, version } = {}) {
        super(registry, decodeExtrinsicPayload(registry, value, version, preamble));
    }
    /**
     * @description The block [[BlockHash]] the signature applies to (mortal/immortal)
     */
    get blockHash() {
        return this.inner.blockHash;
    }
    /**
     * @description The [[ExtrinsicEra]]
     */
    get era() {
        return this.inner.era;
    }
    /**
     * @description The genesis block [[BlockHash]] the signature applies to
     */
    get genesisHash() {
        // NOTE only v3+
        return this.inner.genesisHash || this.registry.createTypeUnsafe('Hash', []);
    }
    /**
     * @description The [[Bytes]] contained in the payload
     */
    get method() {
        return this.inner.method;
    }
    /**
     * @description The [[Index]]
     */
    get nonce() {
        return this.inner.nonce;
    }
    /**
     * @description The specVersion as a [[u32]] for this payload
     */
    get specVersion() {
        // NOTE only v3+
        return this.inner.specVersion || this.registry.createTypeUnsafe('u32', []);
    }
    /**
     * @description The [[Balance]]
     */
    get tip() {
        // NOTE from v2+
        return this.inner.tip || this.registry.createTypeUnsafe('Compact<Balance>', []);
    }
    /**
     * @description The transaction version as a [[u32]] for this payload
     */
    get transactionVersion() {
        // NOTE only v4+
        return this.inner.transactionVersion || this.registry.createTypeUnsafe('u32', []);
    }
    /**
     * @description The (optional) asset id as a [[u32]] or [[MultiLocation]] for this payload
     */
    get assetId() {
        return this.inner.assetId;
    }
    /**
     * @description The (optional) [[Hash]] of the genesis metadata for this payload
     */
    get metadataHash() {
        return this.inner.metadataHash;
    }
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other) {
        return this.inner.eq(other);
    }
    /**
     * @description Sign the payload with the keypair
     */
    sign(signerPair) {
        const signature = this.inner.sign(signerPair);
        // This is extensible, so we could quite readily extend to send back extra
        // information, such as for instance the payload, i.e. `payload: this.toHex()`
        // For the case here we sign via the extrinsic, we ignore the return, so generally
        // this is applicable for external signing
        return {
            signature: u8aToHex(signature)
        };
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended, disableAscii) {
        return this.inner.toHuman(isExtended, disableAscii);
    }
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON() {
        return this.toHex();
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        return 'ExtrinsicPayload';
    }
    /**
     * @description Returns the string representation of the value
     */
    toString() {
        return this.toHex();
    }
    /**
     * @description Returns a serialized u8a form
     */
    toU8a(isBare) {
        // call our parent, with only the method stripped
        return super.toU8a(isBare ? { method: true } : false);
    }
}
