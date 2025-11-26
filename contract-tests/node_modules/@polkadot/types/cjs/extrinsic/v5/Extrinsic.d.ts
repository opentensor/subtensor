import type { HexString } from '@polkadot/util/types';
import type { ExtrinsicSignatureV5 } from '../../interfaces/extrinsics/index.js';
import type { Address, Call } from '../../interfaces/runtime/index.js';
import type { ExtrinsicPayloadValue, IExtrinsicV5Impl, IKeyringPair, Registry, SignatureOptions } from '../../types/index.js';
import type { ExtrinsicOptions, Preamble } from '../types.js';
import { Struct } from '@polkadot/types-codec';
export declare const EXTRINSIC_VERSION = 5;
export interface ExtrinsicValueV5 {
    method?: Call;
    signature?: ExtrinsicSignatureV5;
}
/**
 * @name GenericExtrinsicV5
 * @description
 * The fourth generation of compact extrinsics
 */
export declare class GenericExtrinsicV5 extends Struct implements IExtrinsicV5Impl {
    constructor(registry: Registry, value?: Uint8Array | ExtrinsicValueV5 | Call, { isSigned }?: Partial<ExtrinsicOptions>);
    /** @internal */
    static decodeExtrinsic(registry: Registry, value?: Call | Uint8Array | ExtrinsicValueV5, isSigned?: boolean): ExtrinsicValueV5;
    /**
     * @description The length of the value when encoded as a Uint8Array
     */
    get encodedLength(): number;
    /**
     * @description The [[Call]] this extrinsic wraps
     */
    get method(): Call;
    /**
     * @description The [[ExtrinsicSignatureV5]]
     */
    get signature(): ExtrinsicSignatureV5;
    /**
     * @description The version for the signature
     */
    get version(): number;
    /**
     * @description The [[Preamble]] for the extrinsic
     */
    get preamble(): Preamble;
    /**
     * @description Add an [[ExtrinsicSignatureV5]] to the extrinsic (already generated)
     *
     * [Disabled for ExtrinsicV5]
     */
    addSignature(_signer: Address | Uint8Array | string, _signature: Uint8Array | HexString, _payload: ExtrinsicPayloadValue | Uint8Array | HexString): GenericExtrinsicV5;
    /**
     * @description Sign the extrinsic with a specific keypair
     *
     * [Disabled for ExtrinsicV5]
     */
    sign(_account: IKeyringPair, _options: SignatureOptions): GenericExtrinsicV5;
    /**
     * @describe Adds a fake signature to the extrinsic
     *
     * [Disabled for ExtrinsicV5]
     */
    signFake(_signer: Address | Uint8Array | string, _options: SignatureOptions): GenericExtrinsicV5;
}
