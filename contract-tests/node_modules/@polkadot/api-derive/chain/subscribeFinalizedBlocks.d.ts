import type { Observable } from 'rxjs';
import type { SignedBlockExtended } from '../type/types.js';
import type { DeriveApi } from '../types.js';
/**
 * @name subscribeFinalizedBlocks
 * @description Retrieves the finalized block & events for that block
 * @example
 * ```javascript
 * const unsub = await api.derive.chain.subscribeFinalizedBlocks((finalizedBlock) => {
 *  console.log(`# Finalized block ${finalizedBlock.block.hash}`);
 * });
 * ```
 */
export declare function subscribeFinalizedBlocks(instanceId: string, api: DeriveApi): () => Observable<SignedBlockExtended>;
