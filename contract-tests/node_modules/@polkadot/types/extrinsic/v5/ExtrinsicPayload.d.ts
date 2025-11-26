import type { Hash, MultiLocation } from '@polkadot/types/interfaces';
import type { Bytes } from '@polkadot/types-codec';
import type { Inspect, Registry } from '@polkadot/types-codec/types';
import type { HexString } from '@polkadot/util/types';
import type { BlockHash } from '../../interfaces/chain/index.js';
import type { ExtrinsicEra } from '../../interfaces/extrinsics/index.js';
import type { ExtrinsicPayloadValue, ICompact, IKeyringPair, INumber, IOption } from '../../types/index.js';
import { Struct } from '@polkadot/types-codec';
/**
 * @name GenericExtrinsicPayloadV5
 * @description
 * A signing payload for an [[Extrinsic]]. For the final encoding, it is
 * variable length based on the contents included
 */
export declare class GenericExtrinsicPayloadV5 extends Struct {
    constructor(registry: Registry, value?: ExtrinsicPayloadValue | Uint8Array | HexString);
    /**
     * @description Returns a breakdown of the hex encoding for this Codec
     */
    inspect(): Inspect;
    /**
     * @description The block [[BlockHash]] the signature applies to (mortal/immortal)
     */
    get blockHash(): BlockHash;
    /**
     * @description The [[ExtrinsicEra]]
     */
    get era(): ExtrinsicEra;
    /**
     * @description The genesis [[BlockHash]] the signature applies to (mortal/immortal)
     */
    get genesisHash(): BlockHash;
    /**
     * @description The [[Bytes]] contained in the payload
     */
    get method(): Bytes;
    /**
     * @description The [[Index]]
     */
    get nonce(): ICompact<INumber>;
    /**
     * @description The specVersion for this signature
     */
    get specVersion(): INumber;
    /**
     * @description The tip [[Balance]]
     */
    get tip(): ICompact<INumber>;
    /**
     * @description The transactionVersion for this signature
     */
    get transactionVersion(): INumber;
    /**
     * @description The (optional) asset id for this signature for chains that support transaction fees in assets
     */
    get assetId(): IOption<INumber | MultiLocation>;
    /**
     * @description The (optional) metadataHash proof for the CheckMetadataHash TransactionExtension
     */
    get metadataHash(): IOption<Hash>;
    /**
     * @description Sign the payload with the keypair
     *
     * [Disabled for ExtrinsicV5]
     */
    sign(_signerPair: IKeyringPair): Uint8Array;
}
