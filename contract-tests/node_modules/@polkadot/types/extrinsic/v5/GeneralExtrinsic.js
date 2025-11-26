import { Struct } from '@polkadot/types-codec';
import { compactAddLength, compactFromU8a, isHex, isObject, isU8a, objectSpread, u8aConcat, u8aToHex, u8aToU8a } from '@polkadot/util';
import { EMPTY_U8A, UNMASK_VERSION } from '../constants.js';
function decodeU8a(u8a) {
    if (!u8a.length) {
        return new Uint8Array();
    }
    const [offset, length] = compactFromU8a(u8a);
    const total = offset + length.toNumber();
    if (total > u8a.length) {
        throw new Error(`Extrinsic: length less than remainder, expected at least ${total}, found ${u8a.length}`);
    }
    const data = u8a.subarray(offset, total);
    // 69 denotes 0b01000101 which is the version and preamble for this Extrinsic
    if (data[0] !== 69) {
        throw new Error(`Extrinsic: incorrect version for General Transactions, expected 5, found ${data[0] & UNMASK_VERSION}`);
    }
    return data.subarray(1);
}
export class GeneralExtrinsic extends Struct {
    #version;
    #preamble;
    constructor(registry, value, opt) {
        const extTypes = registry.getSignedExtensionTypes();
        super(registry, objectSpread({
            transactionExtensionVersion: 'u8'
        }, extTypes, {
            method: 'Call'
        }), GeneralExtrinsic.decodeExtrinsic(registry, value));
        this.#version = opt?.version || 0b00000101;
        this.#preamble = 0b01000000;
    }
    static decodeExtrinsic(registry, value) {
        if (!value) {
            return EMPTY_U8A;
        }
        else if (value instanceof GeneralExtrinsic) {
            return value;
        }
        else if (isU8a(value) || Array.isArray(value) || isHex(value)) {
            return decodeU8a(u8aToU8a(value));
        }
        else if (isObject(value)) {
            const { payload, transactionExtensionVersion } = value;
            return objectSpread(payload || {}, {
                transactionExtensionVersion: transactionExtensionVersion || registry.getTransactionExtensionVersion()
            });
        }
        return {};
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return super.encodedLength;
    }
    /**
     * @description The [[ExtrinsicEra]]
     */
    get era() {
        return this.getT('era');
    }
    /**
     * @description The [[Index]]
     */
    get nonce() {
        return this.getT('nonce');
    }
    /**
     * @description The tip [[Balance]]
     */
    get tip() {
        return this.getT('tip');
    }
    /**
     * @description The (optional) asset id for this signature for chains that support transaction fees in assets
     */
    get assetId() {
        return this.getT('assetId');
    }
    /**
     * @description The mode used for the CheckMetadataHash TransactionExtension
     */
    get mode() {
        return this.getT('mode');
    }
    /**
     * @description The (optional) [[Hash]] for the metadata proof
     */
    get metadataHash() {
        return this.getT('metadataHash');
    }
    /**
     * @description The version of the TransactionExtensions used in this extrinsic
     */
    get transactionExtensionVersion() {
        return this.getT('transactionExtensionVersion');
    }
    /**
     * @description The [[Call]] this extrinsic wraps
     */
    get method() {
        return this.getT('method');
    }
    /**
     * @description The extrinsic's version
     */
    get version() {
        return this.#version;
    }
    /**
     * @description The [[Preamble]] for the extrinsic
     */
    get preamble() {
        return this.#preamble;
    }
    toHex(isBare) {
        return u8aToHex(this.toU8a(isBare));
    }
    toU8a(isBare) {
        return isBare
            ? this.encode()
            : compactAddLength(this.encode());
    }
    toRawType() {
        return 'GeneralExt';
    }
    /**
     *
     * @description Returns an encoded GeneralExtrinsic
     */
    encode() {
        return u8aConcat(new Uint8Array([this.version | this.preamble]), super.toU8a());
    }
    signFake() {
        throw new Error('Extrinsic: Type GeneralExtrinsic does not have signFake implemented');
    }
    addSignature() {
        throw new Error('Extrinsic: Type GeneralExtrinsic does not have addSignature implemented');
    }
    sign() {
        throw new Error('Extrinsic: Type GeneralExtrinsic does not have sign implemented');
    }
    signature() {
        throw new Error('Extrinsic: Type GeneralExtrinsic does not have the signature getter');
    }
}
