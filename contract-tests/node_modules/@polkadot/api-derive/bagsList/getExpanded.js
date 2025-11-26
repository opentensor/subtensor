import { map, switchMap } from 'rxjs';
import { objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
/**
 * @name expand
 * @description Expands a given bag by retrieving all its nodes (accounts contained within the bag).
 * @param {Bag} bag The bag to be expanded.
 */
export function expand(instanceId, api) {
    return memo(instanceId, (bag) => api.derive.bagsList.listNodes(bag.bag).pipe(map((nodes) => objectSpread({ nodes }, bag))));
}
/**
 * @name getExpanded
 * @description Retrieves and expands a specific bag from the BagsList pallet.
 * @param {BN | number} id The id of the bag to expand.
 */
export function getExpanded(instanceId, api) {
    return memo(instanceId, (id) => api.derive.bagsList.get(id).pipe(switchMap((bag) => api.derive.bagsList.expand(bag))));
}
