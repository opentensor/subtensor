import { combineLatest, map, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name events
 * @param {Hash} at Block hash to query at.
 * @description Retrieves the block information alongside its events at a given block hash
 * @example
 * ```javascript
 * const blockHash = api.registry.createType(
 *   "Hash",
 *   "0xf1dc2efe8265be67deea5e91b05a98a7f9f81f66854e92825cf36f541beb7af6"
 * );
 * const { events, block } = await api.derive.tx.events(blockHash);
 * ```
 */
export function events(instanceId, api) {
    return memo(instanceId, (blockHash) => combineLatest([
        api.rpc.chain.getBlock(blockHash),
        api.queryAt(blockHash).pipe(switchMap((queryAt) => queryAt.system.events()))
    ]).pipe(map(([block, events]) => ({ block, events }))));
}
