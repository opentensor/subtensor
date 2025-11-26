import { map, of, switchMap } from 'rxjs';
import { BN_ZERO, bnToBn, objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
import { getQueryInterface } from './util.js';
function orderBags(ids, bags) {
    const sorted = ids
        .map((id, index) => ({
        bag: bags[index].unwrapOr(null),
        id,
        key: id.toString()
    }))
        .sort((a, b) => b.id.cmp(a.id));
    const max = sorted.length - 1;
    return sorted.map((entry, index) => objectSpread(entry, {
        bagLower: index === max
            ? BN_ZERO
            : sorted[index + 1].id,
        bagUpper: entry.id,
        index
    }));
}
export function _getIds(instanceId, api) {
    const query = getQueryInterface(api);
    return memo(instanceId, (_ids) => {
        const ids = _ids.map((id) => bnToBn(id));
        return ids.length
            ? query.listBags.multi(ids).pipe(map((bags) => orderBags(ids, bags)))
            : of([]);
    });
}
export function all(instanceId, api) {
    const query = getQueryInterface(api);
    return memo(instanceId, () => query.listBags.keys().pipe(switchMap((keys) => api.derive.bagsList._getIds(keys.map(({ args: [id] }) => id))), map((list) => list.filter(({ bag }) => bag))));
}
/**
 * @name get
 * @param {(BN | number)} id The id of the bag to retrieve.
 * @description Retrieves a specific bag from the BagsList pallet by its id.
 */
export function get(instanceId, api) {
    return memo(instanceId, (id) => api.derive.bagsList._getIds([bnToBn(id)]).pipe(map((bags) => bags[0])));
}
