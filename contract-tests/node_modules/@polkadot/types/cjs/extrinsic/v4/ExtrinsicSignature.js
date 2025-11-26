"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsicSignatureV4 = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const constants_js_1 = require("../constants.js");
const ExtrinsicPayload_js_1 = require("./ExtrinsicPayload.js");
const FAKE_SIGNATURE = new Uint8Array(256).fill(1);
function toAddress(registry, address) {
    return registry.createTypeUnsafe('Address', [(0, util_1.isU8a)(address) ? (0, util_1.u8aToHex)(address) : address]);
}
/**
 * @name GenericExtrinsicSignatureV4
 * @description
 * A container for the [[Signature]] associated with a specific [[Extrinsic]]
 */
class GenericExtrinsicSignatureV4 extends types_codec_1.Struct {
    #signKeys;
    constructor(registry, value, { isSigned } = {}) {
        const signTypes = registry.getSignedExtensionTypes();
        super(registry, (0, util_1.objectSpread)(
        // eslint-disable-next-line sort-keys
        { signer: 'Address', signature: 'ExtrinsicSignature' }, signTypes), GenericExtrinsicSignatureV4.decodeExtrinsicSignature(value, isSigned));
        this.#signKeys = Object.keys(signTypes);
        (0, util_1.objectProperties)(this, this.#signKeys, (k) => this.get(k));
    }
    /** @internal */
    static decodeExtrinsicSignature(value, isSigned = false) {
        if (!value) {
            return constants_js_1.EMPTY_U8A;
        }
        else if (value instanceof GenericExtrinsicSignatureV4) {
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
     * @description The [[Hash]] for the metadata
     */
    get metadataHash() {
        return this.getT('metadataHash');
    }
    _injectSignature(signer, signature, payload) {
        // use the fields exposed to guide the getters
        for (let i = 0, count = this.#signKeys.length; i < count; i++) {
            const k = this.#signKeys[i];
            const v = payload.get(k);
            if (!(0, util_1.isUndefined)(v)) {
                this.set(k, v);
            }
        }
        // additional fields (exposed in struct itself)
        this.set('signer', signer);
        this.set('signature', signature);
        return this;
    }
    /**
     * @description Adds a raw signature
     */
    addSignature(signer, signature, payload) {
        return this._injectSignature(toAddress(this.registry, signer), this.registry.createTypeUnsafe('ExtrinsicSignature', [signature]), new ExtrinsicPayload_js_1.GenericExtrinsicPayloadV4(this.registry, payload));
    }
    /**
     * @description Creates a payload from the supplied options
     */
    createPayload(method, options) {
        const { era, runtimeVersion: { specVersion, transactionVersion } } = options;
        return new ExtrinsicPayload_js_1.GenericExtrinsicPayloadV4(this.registry, (0, util_1.objectSpread)({}, options, {
            era: era || constants_js_1.IMMORTAL_ERA,
            method: method.toHex(),
            specVersion,
            transactionVersion
        }));
    }
    /**
     * @description Generate a payload and applies the signature from a keypair
     */
    sign(method, account, options) {
        if (!account?.addressRaw) {
            throw new Error(`Expected a valid keypair for signing, found ${(0, util_1.stringify)(account)}`);
        }
        const payload = this.createPayload(method, options);
        return this._injectSignature(toAddress(this.registry, account.addressRaw), this.registry.createTypeUnsafe('ExtrinsicSignature', [payload.sign(account)]), payload);
    }
    /**
     * @description Generate a payload and applies a fake signature
     */
    signFake(method, address, options) {
        if (!address) {
            throw new Error(`Expected a valid address for signing, found ${(0, util_1.stringify)(address)}`);
        }
        const payload = this.createPayload(method, options);
        return this._injectSignature(toAddress(this.registry, address), this.registry.createTypeUnsafe('ExtrinsicSignature', [FAKE_SIGNATURE]), payload);
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
exports.GenericExtrinsicSignatureV4 = GenericExtrinsicSignatureV4;
