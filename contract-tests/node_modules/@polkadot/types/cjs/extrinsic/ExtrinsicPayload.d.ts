import type { Bytes } from '@polkadot/types-codec';
import type { AnyJson, BareOpts, Registry } from '@polkadot/types-codec/types';
import type { HexString } from '@polkadot/util/types';
import type { BlockHash } from '../interfaces/chain/index.js';
import type { ExtrinsicPayloadV5 } from '../interfaces/extrinsics/index.js';
import type { Hash, MultiLocation } from '../interfaces/types.js';
import type { ExtrinsicPayloadValue, ICompact, IKeyringPair, INumber, IOption } from '../types/index.js';
import type { GenericExtrinsicEra } from './ExtrinsicEra.js';
import type { Preamble } from './types.js';
import { AbstractBase } from '@polkadot/types-codec';
interface ExtrinsicPayloadOptions {
    version?: number;
    preamble?: Preamble;
}
type ExtrinsicPayloadVx = ExtrinsicPayloadV5;
/**
 * HACK: In order to change the assetId from `number | object` to HexString (While maintaining the true type ie Option<TAssetConversion>),
 * to allow for easier generalization of the SignerPayloadJSON interface the below check is necessary. The ExtrinsicPayloadV4 class does not like
 * a value passed in as an Option, and can't decode it properly. Therefore, we ensure to convert the following below, and then pass the option as a unwrapped
 * JSON value.
 *
 * ref: https://github.com/polkadot-js/api/pull/5968
 * ref: https://github.com/polkadot-js/api/pull/5967
 */
export declare function decodeAssetId(registry: Registry, payload?: ExtrinsicPayloadValue | Uint8Array | HexString): Uint8Array | `0x${string}` | ExtrinsicPayloadValue | {
    assetId: AnyJson;
    blockHash: import("@polkadot/types-codec/types").AnyU8a;
    era: import("@polkadot/types-codec/types").AnyU8a | import("../types/extrinsic.js").IExtrinsicEra;
    genesisHash: import("@polkadot/types-codec/types").AnyU8a;
    method: import("@polkadot/types-codec/types").AnyU8a | import("../types/interfaces.js").IMethod<import("@polkadot/types-codec/types").AnyTuple>;
    nonce: import("@polkadot/types-codec/types").AnyNumber;
    specVersion: import("@polkadot/types-codec/types").AnyNumber;
    tip: import("@polkadot/types-codec/types").AnyNumber;
    transactionVersion: import("@polkadot/types-codec/types").AnyNumber;
    mode?: import("@polkadot/types-codec/types").AnyNumber;
    metadataHash?: import("@polkadot/types-codec/types").AnyU8a;
} | undefined;
/**
 * @name GenericExtrinsicPayload
 * @description
 * A signing payload for an [[Extrinsic]]. For the final encoding, it is variable length based
 * on the contents included
 */
export declare class GenericExtrinsicPayload extends AbstractBase<ExtrinsicPayloadVx> {
    constructor(registry: Registry, value?: Partial<ExtrinsicPayloadValue> | Uint8Array | string, { preamble, version }?: ExtrinsicPayloadOptions);
    /**
     * @description The block [[BlockHash]] the signature applies to (mortal/immortal)
     */
    get blockHash(): BlockHash;
    /**
     * @description The [[ExtrinsicEra]]
     */
    get era(): GenericExtrinsicEra;
    /**
     * @description The genesis block [[BlockHash]] the signature applies to
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
     * @description The specVersion as a [[u32]] for this payload
     */
    get specVersion(): INumber;
    /**
     * @description The [[Balance]]
     */
    get tip(): ICompact<INumber>;
    /**
     * @description The transaction version as a [[u32]] for this payload
     */
    get transactionVersion(): INumber;
    /**
     * @description The (optional) asset id as a [[u32]] or [[MultiLocation]] for this payload
     */
    get assetId(): IOption<INumber | MultiLocation>;
    /**
     * @description The (optional) [[Hash]] of the genesis metadata for this payload
     */
    get metadataHash(): IOption<Hash>;
    /**
     * @description Compares the value of the input to see if there is a match
     */
    eq(other?: unknown): boolean;
    /**
     * @description Sign the payload with the keypair
     */
    sign(signerPair: IKeyringPair): {
        signature: HexString;
    };
    /**
     * @description Converts the Object to to a human-friendly JSON, with additional fields, expansion and formatting of information
     */
    toHuman(isExtended?: boolean, disableAscii?: boolean): AnyJson;
    /**
     * @description Converts the Object to JSON, typically used for RPC transfers
     */
    toJSON(): any;
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType(): string;
    /**
     * @description Returns the string representation of the value
     */
    toString(): string;
    /**
     * @description Returns a serialized u8a form
     */
    toU8a(isBare?: BareOpts): Uint8Array;
}
export {};
