"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.GenericExtrinsicPayloadV5 = void 0;
const types_codec_1 = require("@polkadot/types-codec");
const util_1 = require("@polkadot/util");
/**
 * @name GenericExtrinsicPayloadV5
 * @description
 * A signing payload for an [[Extrinsic]]. For the final encoding, it is
 * variable length based on the contents included
 */
class GenericExtrinsicPayloadV5 extends types_codec_1.Struct {
    constructor(registry, value) {
        super(registry, (0, util_1.objectSpread)({ method: 'Bytes' }, registry.getSignedExtensionTypes(), registry.getSignedExtensionExtra()), value);
    }
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect() {
        return super.inspect({ method: true });
    }
    /**
     * @description The block [[BlockHash]] the signature applies to (mortal/immortal)
     */
    get blockHash() {
        return this.getT('blockHash');
    }
    /**
     * @description The [[ExtrinsicEra]]
     */
    get era() {
        return this.getT('era');
    }
    /**
     * @description The genesis [[BlockHash]] the signature applies to (mortal/immortal)
     */
    get genesisHash() {
        return this.getT('genesisHash');
    }
    /**
     * @description The [[Bytes]] contained in the payload
     */
    get method() {
        return this.getT('method');
    }
    /**
     * @description The [[Index]]
     */
    get nonce() {
        return this.getT('nonce');
    }
    /**
     * @description The specVersion for this signature
     */
    get specVersion() {
        return this.getT('specVersion');
    }
    /**
     * @description The tip [[Balance]]
     */
    get tip() {
        return this.getT('tip');
    }
    /**
     * @description The transactionVersion for this signature
     */
    get transactionVersion() {
        return this.getT('transactionVersion');
    }
    /**
     * @description The (optional) asset id for this signature for chains that support transaction fees in assets
     */
    get assetId() {
        return this.getT('assetId');
    }
    /**
     * @description The (optional) metadataHash proof for the CheckMetadataHash TransactionExtension
     */
    get metadataHash() {
        return this.getT('metadataHash');
    }
    /**
     * @description Sign the payload with the keypair
     *
     * [Disabled for ExtrinsicV5]
     */
    sign(_signerPair) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
}
exports.GenericExtrinsicPayloadV5 = GenericExtrinsicPayloadV5;
