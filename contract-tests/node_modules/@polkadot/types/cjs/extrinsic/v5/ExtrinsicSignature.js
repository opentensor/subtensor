"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsicSignatureV5 = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const constants_js_1 = require("../constants.js");
const ExtrinsicPayload_js_1 = require("./ExtrinsicPayload.js");
/**
 * @name GenericExtrinsicSignatureV5
 * @description
 * A container for the [[Signature]] associated with a specific [[Extrinsic]]
 */
class GenericExtrinsicSignatureV5 extends types_codec_1.Struct {
    #signKeys;
    constructor(registry, value, { isSigned } = {}) {
        const signTypes = registry.getSignedExtensionTypes();
        super(registry, (0, util_1.objectSpread)(
        // eslint-disable-next-line sort-keys
        { signer: 'Address', signature: 'ExtrinsicSignature', transactionExtensionVersion: 'u8' }, signTypes), GenericExtrinsicSignatureV5.decodeExtrinsicSignature(value, isSigned));
        this.#signKeys = Object.keys(signTypes);
        (0, util_1.objectProperties)(this, this.#signKeys, (k) => this.get(k));
    }
    /** @internal */
    static decodeExtrinsicSignature(value, isSigned = false) {
        if (!value) {
            return constants_js_1.EMPTY_U8A;
        }
        else if (value instanceof GenericExtrinsicSignatureV5) {
            return value;
        }
        return isSigned
            ? value
            : constants_js_1.EMPTY_U8A;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.isSigned
            ? super.encodedLength
            : 0;
    }
    /**
     * @description `true` if the signature is valid
     */
    get isSigned() {
        return !this.signature.isEmpty;
    }
    /**
     * @description The [[ExtrinsicEra]] (mortal or immortal) this signature applies to
     */
    get era() {
        return this.getT('era');
    }
    /**
     * @description The [[Index]] for the signature
     */
    get nonce() {
        return this.getT('nonce');
    }
    /**
     * @description The actual [[EcdsaSignature]], [[Ed25519Signature]] or [[Sr25519Signature]]
     */
    get signature() {
        // the second case here is when we don't have an enum signature, treat as raw
        return (this.multiSignature.value || this.multiSignature);
    }
    /**
     * @description The raw [[ExtrinsicSignature]]
     */
    get multiSignature() {
        return this.getT('signature');
    }
    /**
     * @description The [[Address]] that signed
     */
    get signer() {
        return this.getT('signer');
    }
    /**
     * @description The [[Balance]] tip
     */
    get tip() {
        return this.getT('tip');
    }
    /**
     * @description The [[u32]] or [[MultiLocation]] assetId
     */
    get assetId() {
        return this.getT('assetId');
    }
    /**
     * @description the [[u32]] mode
     */
    get mode() {
        return this.getT('mode');
    }
    /**
     * @description The (optional)  [[Hash]] for the metadata proof
     */
    get metadataHash() {
        return this.getT('metadataHash');
    }
    /**
     * @description The [[u8]] for the TransactionExtension version
     */
    get transactionExtensionVersion() {
        return this.getT('transactionExtensionVersion');
    }
    /**
     * [Disabled for ExtrinsicV5]
     */
    _injectSignature(_signer, _signature, _payload) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
    /**
     * @description Adds a raw signature
     *
     * [Disabled for ExtrinsicV5]
     */
    addSignature(_signer, _signature, _payload) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
    /**
     * @description Creates a payload from the supplied options
     */
    createPayload(method, options) {
        const { era, runtimeVersion: { specVersion, transactionVersion } } = options;
        return new ExtrinsicPayload_js_1.GenericExtrinsicPayloadV5(this.registry, (0, util_1.objectSpread)({}, options, {
            era: era || constants_js_1.IMMORTAL_ERA,
            method: method.toHex(),
            specVersion,
            transactionVersion
        }));
    }
    /**
     * @description Generate a payload and applies the signature from a keypair
     *
     * [Disabled for ExtrinsicV5]
     */
    sign(_method, _account, _options) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
    /**
     * @description Generate a payload and applies a fake signature
     *
     * [Disabled for ExtrinsicV5]
     */
    signFake(_method, _address, _options) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare) {
        return this.isSigned
            ? super.toU8a(isBare)
            : constants_js_1.EMPTY_U8A;
    }
}
exports.GenericExtrinsicSignatureV5 = GenericExtrinsicSignatureV5;
