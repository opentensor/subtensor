import { combineLatest, map, of, switchMap } from 'rxjs';
import { objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
import { didUpdateToBool } from './util.js';
function parse([ids, didUpdate, relayDispatchQueueSizes, infos, pendingSwaps]) {
    return ids.map((id, index) => ({
        didUpdate: didUpdateToBool(didUpdate, id),
        id,
        info: objectSpread({ id }, infos[index].unwrapOr(null)),
        pendingSwapId: pendingSwaps[index].unwrapOr(null),
        relayDispatchQueueSize: relayDispatchQueueSizes[index][0].toNumber()
    }));
}
/**
 * @name overview
 * @description Retrieves an overview of all registered parachains.
 * @example
 * ```javascript
 * await api.derive.parachains.overview((overview) => {
 *   parachains.forEach(parachain => {
 *     console.log(`Parachain ${parachain.id.toString()} is registered.`);
 *   });
 * });
 * ```
 */
export function overview(instanceId, api) {
    return memo(instanceId, () => api.query['registrar']?.['parachains'] && api.query['parachains']
        ? api.query['registrar']['parachains']().pipe(switchMap((paraIds) => combineLatest([
            of(paraIds),
            api.query['parachains']['didUpdate'](),
            api.query['parachains']['relayDispatchQueueSize'].multi(paraIds),
            api.query['registrar']['paras'].multi(paraIds),
            api.query['registrar']['pendingSwap'].multi(paraIds)
        ])), map(parse))
        : of([]));
}
