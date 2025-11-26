import type { MultiLocation } from '@polkadot/types/interfaces';
import type { HexString } from '@polkadot/util/types';
import type { EcdsaSignature, Ed25519Signature, ExtrinsicEra, ExtrinsicSignature, Sr25519Signature } from '../../interfaces/extrinsics/index.js';
import type { Address, Call, Hash } from '../../interfaces/runtime/index.js';
import type { ExtrinsicPayloadValue, ICompact, IExtrinsicSignature, IKeyringPair, INumber, IOption, Registry, SignatureOptions } from '../../types/index.js';
import type { ExtrinsicSignatureOptions } from '../types.js';
import { Struct } from '@polkadot/types-codec';
import { GenericExtrinsicPayloadV5 } from './ExtrinsicPayload.js';
/**
 * @name GenericExtrinsicSignatureV5
 * @description
 * A container for the [[Signature]] associated with a specific [[Extrinsic]]
 */
export declare class GenericExtrinsicSignatureV5 extends Struct implements IExtrinsicSignature {
    #private;
    constructor(registry: Registry, value?: GenericExtrinsicSignatureV5 | Uint8Array, { isSigned }?: ExtrinsicSignatureOptions);
    /** @internal */
    static decodeExtrinsicSignature(value?: GenericExtrinsicSignatureV5 | Uint8Array, isSigned?: boolean): GenericExtrinsicSignatureV5 | Uint8Array;
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description `true` if the signature is valid
     */
    get isSigned(): boolean;
    /**
     * @description The [[ExtrinsicEra]] (mortal or immortal) this signature applies to
     */
    get era(): ExtrinsicEra;
    /**
     * @description The [[Index]] for the signature
     */
    get nonce(): ICompact<INumber>;
    /**
     * @description The actual [[EcdsaSignature]], [[Ed25519Signature]] or [[Sr25519Signature]]
     */
    get signature(): EcdsaSignature | Ed25519Signature | Sr25519Signature;
    /**
     * @description The raw [[ExtrinsicSignature]]
     */
    get multiSignature(): ExtrinsicSignature;
    /**
     * @description The [[Address]] that signed
     */
    get signer(): Address;
    /**
     * @description The [[Balance]] tip
     */
    get tip(): ICompact<INumber>;
    /**
     * @description The [[u32]] or [[MultiLocation]] assetId
     */
    get assetId(): IOption<INumber | MultiLocation>;
    /**
     * @description the [[u32]] mode
     */
    get mode(): INumber;
    /**
     * @description The (optional)  [[Hash]] for the metadata proof
     */
    get metadataHash(): IOption<Hash>;
    /**
     * @description The [[u8]] for the TransactionExtension version
     */
    get transactionExtensionVersion(): INumber;
    /**
     * [Disabled for ExtrinsicV5]
     */
    protected _injectSignature(_signer: Address, _signature: ExtrinsicSignature, _payload: GenericExtrinsicPayloadV5): IExtrinsicSignature;
    /**
     * @description Adds a raw signature
     *
     * [Disabled for ExtrinsicV5]
     */
    addSignature(_signer: Address | Uint8Array | string, _signature: Uint8Array | HexString, _payload: ExtrinsicPayloadValue | Uint8Array | HexString): IExtrinsicSignature;
    /**
     * @description Creates a payload from the supplied options
     */
    createPayload(method: Call, options: SignatureOptions): GenericExtrinsicPayloadV5;
    /**
     * @description Generate a payload and applies the signature from a keypair
     *
     * [Disabled for ExtrinsicV5]
     */
    sign(_method: Call, _account: IKeyringPair, _options: SignatureOptions): IExtrinsicSignature;
    /**
     * @description Generate a payload and applies a fake signature
     *
     * [Disabled for ExtrinsicV5]
     */
    signFake(_method: Call, _address: Address | Uint8Array | string, _options: SignatureOptions): IExtrinsicSignature;
    /**
     * @description Encodes the value as a Uint8Array as per the SCALE specifications
     * @param isBare true when the value has none of the type-specific prefixes (internal)
     */
    toU8a(isBare?: boolean): Uint8Array;
}
