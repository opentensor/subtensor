import type { Observable } from 'rxjs';
import type { Hash } from '@polkadot/types/interfaces';
import type { FrameSupportPreimagesBounded } from '@polkadot/types/lookup';
import type { DeriveApi, DeriveProposalImage } from '../types.js';
/**
 * @name preimages
 * @description Retrieves the full details (preimages) of governance proposals using their on-chain hashes.
 * @param { (Hash | Uint8Array | string | FrameSupportPreimagesBounded)[] } hashes An array of hashes representing governance proposals.
 * @example
 * ```javascript
 * const preimages = await api.derive.democracy.preimages([HASH1, HASH2]);
 * ```
 */
export declare function preimages(instanceId: string, api: DeriveApi): (hashes: (Hash | Uint8Array | string | FrameSupportPreimagesBounded)[]) => Observable<(DeriveProposalImage | undefined)[]>;
/**
 * @name preimage
 * @description Retrieves the full details (preimage) of a governance proposal using its on-chain hash.
 * @param { Hash | Uint8Array | string | FrameSupportPreimagesBounded } hash Hash that represents governance proposals.
 *  * @example
 * ```javascript
 * const preimage = await api.derive.democracy.preimage(HASH);
 * ```
 */
export declare const preimage: (instanceId: string, api: DeriveApi) => (hash: string | Uint8Array | Hash | FrameSupportPreimagesBounded) => Observable<DeriveProposalImage | undefined>;
