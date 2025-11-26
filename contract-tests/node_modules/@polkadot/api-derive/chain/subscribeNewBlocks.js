import { switchMap } from 'rxjs';
import { memo } from '../util/index.js';
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
export function subscribeNewBlocks(instanceId, api) {
    return memo(instanceId, () => api.derive.chain.subscribeNewHeads().pipe(switchMap((header) => api.derive.chain.getBlock(header.createdAtHash || header.hash))));
}
