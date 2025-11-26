import { switchMap } from 'rxjs';
import { memo } from '../util/index.js';
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
export function subscribeFinalizedBlocks(instanceId, api) {
    return memo(instanceId, () => api.derive.chain.subscribeFinalizedHeads().pipe(switchMap((header) => api.derive.chain.getBlock(header.createdAtHash || header.hash))));
}
