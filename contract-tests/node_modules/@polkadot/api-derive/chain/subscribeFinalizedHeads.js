import { from, of, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * Returns a header range from startHash to to (not including) endHash, i.e. lastBlock.parentHash === endHash
 */
export function _getHeaderRange(instanceId, api) {
    return memo(instanceId, (startHash, endHash, prev = []) => api.rpc.chain.getHeader(startHash).pipe(switchMap((header) => header.parentHash.eq(endHash)
        ? of([header, ...prev])
        : api.derive.chain._getHeaderRange(header.parentHash, endHash, [header, ...prev]))));
}
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
export function subscribeFinalizedHeads(instanceId, api) {
    return memo(instanceId, () => {
        let prevHash = null;
        return api.rpc.chain.subscribeFinalizedHeads().pipe(switchMap((header) => {
            const endHash = prevHash;
            const startHash = header.parentHash;
            prevHash = header.createdAtHash = header.hash;
            return endHash === null || startHash.eq(endHash)
                ? of(header)
                : api.derive.chain._getHeaderRange(startHash, endHash, [header]).pipe(switchMap((headers) => from(headers)));
        }));
    });
}
