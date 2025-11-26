import type { Text, Vec } from '@polkadot/types-codec';
import type { Registry } from '@polkadot/types-codec/types';
import type { HexString } from '@polkadot/util/types';
import type { Address, BlockHash, Call, ExtrinsicEra, Hash, MultiLocation } from '../interfaces/index.js';
import type { Codec, ICompact, INumber, IOption, IRuntimeVersion, ISignerPayload, SignerPayloadJSON, SignerPayloadRaw } from '../types/index.js';
import { Struct } from '@polkadot/types-codec';
export interface SignerPayloadType extends Codec {
    address: Address;
    assetId: IOption<INumber | MultiLocation>;
    blockHash: Hash;
    blockNumber: INumber;
    era: ExtrinsicEra;
    genesisHash: Hash;
    metadataHash: IOption<Hash>;
    method: Call;
    mode: INumber;
    nonce: ICompact<INumber>;
    runtimeVersion: IRuntimeVersion;
    signedExtensions: Vec<Text>;
    tip: ICompact<INumber>;
    version: INumber;
}
/**
 * @name GenericSignerPayload
 * @description
 * A generic signer payload that can be used for serialization between API and signer
 */
export declare class GenericSignerPayload extends Struct implements ISignerPayload, SignerPayloadType {
    #private;
    constructor(registry: Registry, value?: HexString | Record<string, unknown> | Map<unknown, unknown> | unknown[]);
    get address(): Address;
    get blockHash(): BlockHash;
    get blockNumber(): INumber;
    get era(): ExtrinsicEra;
    get genesisHash(): BlockHash;
    get method(): Call;
    get nonce(): ICompact<INumber>;
    get runtimeVersion(): IRuntimeVersion;
    get signedExtensions(): Vec<Text>;
    get tip(): ICompact<INumber>;
    get assetId(): IOption<INumber | MultiLocation>;
    get version(): INumber;
    get mode(): INumber;
    get metadataHash(): IOption<Hash>;
    get withSignedTransaction(): boolean;
    /**
     * @description Creates an representation of the structure as an ISignerPayload JSON
     */
    toPayload(): SignerPayloadJSON;
    /**
     * @description Creates a representation of the payload in raw Exrinsic form
     */
    toRaw(): SignerPayloadRaw;
}
