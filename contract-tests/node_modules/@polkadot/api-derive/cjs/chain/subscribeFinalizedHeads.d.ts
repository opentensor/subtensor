import type { Observable } from 'rxjs';
import type { Hash, Header } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
/**
 * Returns a header range from startHash to to (not including) endHash, i.e. lastBlock.parentHash === endHash
 */
export declare function _getHeaderRange(instanceId: string, api: DeriveApi): (startHash: Hash, endHash: Hash, prev?: Header[]) => Observable<Header[]>;
/**
 * @name subscribeFinalizedHeads
 * @description An observable of the finalized block headers. Unlike the base
 * chain.subscribeFinalizedHeads this does not skip any headers. Since finalization
 * may skip specific blocks (finalization happens in terms of chains), this version
 * of the derive tracks missing headers (since last retrieved) and provides them
 * to the caller.
 * @example
 * ```javascript
 * const unsub = await api.derive.chain.subscribeFinalizedHeads((finalizedHead) => {
 *   console.log(`${finalizedHead.hash}`);
 * });
 * ```
 */
export declare function subscribeFinalizedHeads(instanceId: string, api: DeriveApi): () => Observable<Header>;
