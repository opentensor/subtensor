import type { Observable } from 'rxjs';
import type { SignedBlockExtended } from '../type/types.js';
import type { DeriveApi } from '../types.js';
/**
 * @name subscribeNewBlocks
 * @returns The latest block & events for that block
 * @example
 * ```javascript
 * const unsub = await api.derive.chain.subscribeNewBlocks((newBlock) => {
 *   console.log(`Block Hash: ${newBlock.hash}`);
 * });
 * ```
 */
export declare function subscribeNewBlocks(instanceId: string, api: DeriveApi): () => Observable<SignedBlockExtended>;
