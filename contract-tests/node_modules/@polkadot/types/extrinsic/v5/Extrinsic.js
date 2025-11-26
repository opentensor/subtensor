import { Struct } from '@polkadot/types-codec';
import { isU8a } from '@polkadot/util';
export const EXTRINSIC_VERSION = 5;
/**
 * @name GenericExtrinsicV5
 * @description
 * The fourth generation of compact extrinsics
 */
export class GenericExtrinsicV5 extends Struct {
    constructor(registry, value, { isSigned } = {}) {
        super(registry, {
            signature: 'ExtrinsicSignatureV5',
            // eslint-disable-next-line sort-keys
            method: 'Call'
        }, GenericExtrinsicV5.decodeExtrinsic(registry, value, isSigned));
    }
    /** @internal */
    static decodeExtrinsic(registry, value, isSigned = false) {
        if (value instanceof GenericExtrinsicV5) {
            return value;
        }
        else if (value instanceof registry.createClassUnsafe('Call')) {
            return { method: value };
        }
        else if (isU8a(value)) {
            // here we decode manually since we need to pull through the version information
            const signature = registry.createTypeUnsafe('ExtrinsicSignatureV5', [value, { isSigned }]);
            // We add 2 here since the length of the TransactionExtension Version needs to be accounted for
            const method = registry.createTypeUnsafe('Call', [value.subarray(signature.encodedLength)]);
            return {
                method,
                signature
            };
        }
        return value || {};
    }
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength() {
        return this.toU8a().length;
    }
    /**
     * @description The [[Call]] this extrinsic wraps
     */
    get method() {
        return this.getT('method');
    }
    /**
     * @description The [[ExtrinsicSignatureV5]]
     */
    get signature() {
        return this.getT('signature');
    }
    /**
     * @description The version for the signature
     */
    get version() {
        return EXTRINSIC_VERSION;
    }
    /**
     * @description The [[Preamble]] for the extrinsic
     */
    get preamble() {
        return this.getT('preamble');
    }
    /**
     * @description Add an [[ExtrinsicSignatureV5]] to the extrinsic (already generated)
     *
     * [Disabled for ExtrinsicV5]
     */
    addSignature(_signer, _signature, _payload) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
    /**
     * @description Sign the extrinsic with a specific keypair
     *
     * [Disabled for ExtrinsicV5]
     */
    sign(_account, _options) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
    /**
     * @describe Adds a fake signature to the extrinsic
     *
     * [Disabled for ExtrinsicV5]
     */
    signFake(_signer, _options) {
        throw new Error('Extrinsic: ExtrinsicV5 does not include signing support');
    }
}
