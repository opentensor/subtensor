"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsic = exports.LATEST_EXTRINSIC_VERSION = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
const constants_js_1 = require("./constants.js");
Object.defineProperty(exports, "LATEST_EXTRINSIC_VERSION", { enumerable: true, get: function () { return constants_js_1.LATEST_EXTRINSIC_VERSION; } });
const VERSIONS = [
    'ExtrinsicUnknown', // v0 is unknown
    'ExtrinsicUnknown',
    'ExtrinsicUnknown',
    'ExtrinsicUnknown',
    'ExtrinsicV4',
    'ExtrinsicV5'
];
const PREAMBLE = {
    bare: 'ExtrinsicV5',
    general: 'GeneralExtrinsic'
};
const PreambleMask = {
    bare: constants_js_1.BARE_EXTRINSIC,
    general: constants_js_1.GENERAL_EXTRINSIC
};
const preambleUnMask = {
    0: 'bare',
    // eslint-disable-next-line sort-keys
    64: 'general'
};
/** @internal */
function newFromValue(registry, value, version, preamble) {
    if (value instanceof GenericExtrinsic) {
        return value.unwrap();
    }
    const isSigned = (version & constants_js_1.BIT_SIGNED) === constants_js_1.BIT_SIGNED;
    const type = (version & constants_js_1.VERSION_MASK) === 5 ? PREAMBLE[preamble] : VERSIONS[version & constants_js_1.VERSION_MASK] || VERSIONS[0];
    // we cast here since the VERSION definition is incredibly broad - we don't have a
    // slice for "only add extrinsic types", and more string definitions become unwieldy
    return registry.createTypeUnsafe(type, [value, { isSigned, version }]);
}
/** @internal */
function decodeExtrinsic(registry, value, version = constants_js_1.LOWEST_SUPPORTED_EXTRINSIC_FORMAT_VERSION, preamble = constants_js_1.DEFAULT_PREAMBLE) {
    if ((0, util_1.isU8a)(value) || Array.isArray(value) || (0, util_1.isHex)(value)) {
        return decodeU8a(registry, (0, util_1.u8aToU8a)(value), version, preamble);
    }
    else if (value instanceof registry.createClassUnsafe('Call')) {
        return newFromValue(registry, { method: value }, version, preamble);
    }
    return newFromValue(registry, value, version, preamble);
}
/** @internal */
function decodeU8a(registry, value, version, preamble) {
    if (!value.length) {
        return newFromValue(registry, new Uint8Array(), version, preamble);
    }
    const [offset, length] = (0, util_1.compactFromU8a)(value);
    const total = offset + length.toNumber();
    if (total > value.length) {
        throw new Error(`Extrinsic: length less than remainder, expected at least ${total}, found ${value.length}`);
    }
    const data = value.subarray(offset, total);
    const unmaskedPreamble = data[0] & constants_js_1.TYPE_MASK;
    if (preambleUnMask[`${unmaskedPreamble}`] === 'general') {
        // NOTE: GeneralExtrinsic needs to have the full data to validate the preamble and version
        return newFromValue(registry, value, data[0], preambleUnMask[`${unmaskedPreamble}`] || preamble);
    }
    else {
        return newFromValue(registry, data.subarray(1), data[0], preambleUnMask[`${unmaskedPreamble}`] || preamble);
    }
}
class ExtrinsicBase extends types_codec_1.AbstractBase {
    #preamble;
    constructor(registry, value, initialU8aLength, preamble) {
        super(registry, value, initialU8aLength);
        const signKeys = Object.keys(registry.getSignedExtensionTypes());
        if (this.version === 5 && preamble !== 'general') {
            const getter = (key) => this.inner.signature[key];
            // This is on the abstract class, ensuring that hasOwnProperty operates
            // correctly, i.e. it needs to be on the base class exposing it
            for (let i = 0, count = signKeys.length; i < count; i++) {
                (0, util_1.objectProperty)(this, signKeys[i], getter);
            }
        }
        const unmaskedPreamble = this.type & constants_js_1.TYPE_MASK;
        this.#preamble = preamble || preambleUnMask[`${unmaskedPreamble}`];
    }
    isGeneral() {
        return this.#preamble === 'general';
    }
    /**
     * @description The arguments passed to for the call, exposes args so it is compatible with [[Call]]
     */
    get args() {
        return this.method.args;
    }
    /**
     * @description The argument definitions, compatible with [[Call]]
     */
    get argsDef() {
        return this.method.argsDef;
    }
    /**
     * @description The actual `[sectionIndex, methodIndex]` as used in the Call
     */
    get callIndex() {
        return this.method.callIndex;
    }
    /**
     * @description The actual data for the Call
     */
    get data() {
        return this.method.data;
    }
    /**
     * @description The era for this extrinsic
     */
    get era() {
        return this.isGeneral()
            ? this.inner.era
            : this.inner.signature.era;
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.toU8a().length;
    }
    /**
     * @description `true` id the extrinsic is signed
     */
    get isSigned() {
        return this.isGeneral()
            ? false
            : this.inner.signature.isSigned;
    }
    /**
     * @description The length of the actual data, excluding prefix
     */
    get length() {
        return this.toU8a(true).length;
    }
    /**
     * @description The [[FunctionMetadataLatest]] that describes the extrinsic
     */
    get meta() {
        return this.method.meta;
    }
    /**
     * @description The [[Call]] this extrinsic wraps
     */
    get method() {
        return this.inner.method;
    }
    /**
     * @description The nonce for this extrinsic
     */
    get nonce() {
        return this.isGeneral()
            ? this.inner.nonce
            : this.inner.signature.nonce;
    }
    /**
     * @description The actual [[EcdsaSignature]], [[Ed25519Signature]] or [[Sr25519Signature]]
     */
    get signature() {
        if (this.isGeneral()) {
            throw new Error('Extrinsic: GeneralExtrinsic does not have signature implemented');
        }
        return this.inner.signature.signature;
    }
    /**
     * @description The [[Address]] that signed
     */
    get signer() {
        if (this.isGeneral()) {
            throw new Error('Extrinsic: GeneralExtrinsic does not have signer implemented');
        }
        return this.inner.signature.signer;
    }
    /**
     * @description Forwards compat
     */
    get tip() {
        return this.isGeneral()
            ? this.inner.tip
            : this.inner.signature.tip;
    }
    /**
     * @description Forward compat
     */
    get assetId() {
        return this.isGeneral()
            ? this.inner.assetId
            : this.inner.signature.assetId;
    }
    /**
     * @description Forward compat
     */
    get metadataHash() {
        return this.isGeneral()
            ? this.inner.metadataHash
            : this.inner.signature.metadataHash;
    }
    /**
     * @description Forward compat
     */
    get mode() {
        return this.isGeneral()
            ? this.inner.mode
            : this.inner.signature.mode;
    }
    /**
     * @description Returns the raw transaction version (not flagged with signing information)
    */
    get type() {
        return this.inner.version;
    }
    get inner() {
        return this.unwrap();
    }
    /**
     * @description Returns the encoded version flag
    */
    get version() {
        if (this.type <= constants_js_1.LOWEST_SUPPORTED_EXTRINSIC_FORMAT_VERSION) {
            return this.type | (this.isSigned ? constants_js_1.BIT_SIGNED : constants_js_1.BIT_UNSIGNED);
        }
        else {
            if (this.isSigned) {
                throw new Error('Signed Extrinsics are currently only available for ExtrinsicV4');
            }
            return this.type | (this.isGeneral() ? PreambleMask.general : PreambleMask.bare);
        }
    }
    /**
     * @description Checks if the source matches this in type
     */
    is(other) {
        return this.method.is(other);
    }
    unwrap() {
        return super.unwrap();
    }
}
/**
 * @name GenericExtrinsic
 * @description
 * Representation of an Extrinsic in the system. It contains the actual call,
 * (optional) signature and encodes with an actual length prefix
 *
 * {@link https://github.com/paritytech/wiki/blob/master/Extrinsic.md#the-extrinsic-format-for-node}.
 *
 * Can be:
 * - signed, to create a transaction
 * - left as is, to create an inherent
 */
class GenericExtrinsic extends ExtrinsicBase {
    #hashCache;
    static LATEST_EXTRINSIC_VERSION = constants_js_1.LATEST_EXTRINSIC_VERSION;
    constructor(registry, value, { preamble, version } = {}) {
        const versionsLength = registry.metadata.extrinsic.versions.length;
        // TODO: Once ExtrinsicV5 is fully supported update this to use the highest supported verion which is the last item of the array
        const supportedVersion = versionsLength ? registry.metadata.extrinsic.versions[0] : undefined;
        super(registry, decodeExtrinsic(registry, value, version || supportedVersion, preamble), undefined, preamble);
    }
    /**
     * @description returns a hash of the contents
     */
    get hash() {
        if (!this.#hashCache) {
            this.#hashCache = super.hash;
        }
        return this.#hashCache;
    }
    /**
     * @description Injects an already-generated signature into the extrinsic
     */
    addSignature(signer, signature, payload) {
        this.inner.addSignature(signer, signature, payload);
        this.#hashCache = undefined;
        return this;
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        const encoded = (0, util_1.u8aConcat)(...this.toU8aInner());
        return {
            inner: this.isSigned
                ? this.inner.inspect().inner
                : this.inner.method.inspect().inner,
            outer: [(0, util_1.compactToU8a)(encoded.length), new Uint8Array([this.version])]
        };
    }
    /**
     * @description Sign the extrinsic with a specific keypair
     */
    sign(account, options) {
        this.inner.sign(account, options);
        this.#hashCache = undefined;
        return this;
    }
    /**
     * @describe Adds a fake signature to the extrinsic
     */
    signFake(signer, options) {
        this.inner.signFake(signer, options);
        this.#hashCache = undefined;
        return this;
    }
    /**
     * @description Returns a hex string representation of the value
     */
    toHex(isBare) {
        return (0, util_1.u8aToHex)(this.toU8a(isBare));
    }
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExpanded, disableAscii) {
        return (0, util_1.objectSpread)({}, {
            isSigned: this.isSigned,
            method: this.method.toHuman(isExpanded, disableAscii)
        }, this.isSigned
            ? {
                assetId: this.assetId ? this.assetId.toHuman(isExpanded, disableAscii) : null,
                era: this.era.toHuman(isExpanded, disableAscii),
                metadataHash: this.metadataHash ? this.metadataHash.toHex() : null,
                mode: this.mode ? this.mode.toHuman() : null,
                nonce: this.nonce.toHuman(isExpanded, disableAscii),
                signature: this.signature.toHex(),
                signer: this.signer.toHuman(isExpanded, disableAscii),
                tip: this.tip.toHuman(isExpanded, disableAscii)
            }
            : null);
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
        return 'Extrinsic';
    }
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value is not length-prefixed
     */
    toU8a(isBare) {
        const encoded = (0, util_1.u8aConcat)(...this.toU8aInner());
        return isBare
            ? encoded
            : (0, util_1.compactAddLength)(encoded);
    }
    toU8aInner() {
        // we do not apply bare to the internal values, rather this only determines out length addition,
        // where we strip all lengths this creates an extrinsic that cannot be decoded
        return [
            new Uint8Array([this.version]),
            this.inner.toU8a()
        ];
    }
}
exports.GenericExtrinsic = GenericExtrinsic;
