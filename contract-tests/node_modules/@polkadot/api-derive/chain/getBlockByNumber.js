import { switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name getBlockByNumber
 * @param {( BN | bigint | Uint8Array | number | string )} blockNumber
 * @description Get a specific block (e.g. rpc.chain.getBlock) and extend it with the author by block number
 * @example
 * ```javascript
 * const { author, block } = await api.derive.chain.getBlockByNumber(123);
 *
 * console.log(`block #${block.header.number} was authored by ${author}`);
 * ```
 */
export function getBlockByNumber(instanceId, api) {
    return memo(instanceId, (blockNumber) => api.rpc.chain.getBlockHash(blockNumber).pipe(switchMap((h) => api.derive.chain.getBlock(h))));
}
