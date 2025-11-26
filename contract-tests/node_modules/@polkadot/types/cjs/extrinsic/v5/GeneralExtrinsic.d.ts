import type { Call, ExtrinsicEra, Hash, MultiLocation } from '@polkadot/types/interfaces';
import type { AnyNumber, AnyU8a, ICompact, IExtrinsicEra, INumber, IOption, Registry } from '@polkadot/types/types';
import type { AnyTuple, IMethod } from '@polkadot/types-codec/types';
import type { HexString } from '@polkadot/util/types';
import { Struct } from '@polkadot/types-codec';
interface TransactionExtensionValues {
    era: AnyU8a | IExtrinsicEra;
    nonce: AnyNumber;
    tip: AnyNumber;
    transactionVersion: AnyNumber;
    assetId?: HexString;
    mode?: AnyNumber;
    metadataHash?: AnyU8a;
}
interface GeneralExtrinsicPayloadValues extends TransactionExtensionValues {
    method: AnyU8a | IMethod<AnyTuple>;
}
interface GeneralExtrinsicValue {
    payload?: GeneralExtrinsicPayloadValues;
    transactionExtensionVersion?: number;
}
export declare class GeneralExtrinsic extends Struct {
    #private;
    constructor(registry: Registry, value?: GeneralExtrinsicValue | Uint8Array | HexString, opt?: {
        version: number;
    });
    static decodeExtrinsic(registry: Registry, value?: GeneralExtrinsicValue | Uint8Array | HexString): object;
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description The [[ExtrinsicEra]]
     */
    get era(): ExtrinsicEra;
    /**
     * @description The [[Index]]
     */
    get nonce(): ICompact<INumber>;
    /**
     * @description The tip [[Balance]]
     */
    get tip(): ICompact<INumber>;
    /**
     * @description The (optional) asset id for this signature for chains that support transaction fees in assets
     */
    get assetId(): IOption<INumber | MultiLocation>;
    /**
     * @description The mode used for the CheckMetadataHash TransactionExtension
     */
    get mode(): INumber;
    /**
     * @description The (optional) [[Hash]] for the metadata proof
     */
    get metadataHash(): IOption<Hash>;
    /**
     * @description The version of the TransactionExtensions used in this extrinsic
     */
    get transactionExtensionVersion(): INumber;
    /**
     * @description The [[Call]] this extrinsic wraps
     */
    get method(): Call;
    /**
     * @description The extrinsic's version
     */
    get version(): number;
    /**
     * @description The [[Preamble]] for the extrinsic
     */
    get preamble(): number;
    toHex(isBare?: boolean): HexString;
    toU8a(isBare?: boolean): Uint8Array;
    toRawType(): string;
    /**
     *
     * @description Returns an encoded GeneralExtrinsic
     */
    encode(): Uint8Array;
    signFake(): void;
    addSignature(): void;
    sign(): void;
    signature(): void;
}
export {};
