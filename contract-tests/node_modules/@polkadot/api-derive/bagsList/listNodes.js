import { BehaviorSubject, map, of, switchMap, tap, toArray } from 'rxjs';
import { nextTick } from '@polkadot/util';
import { memo } from '../util/index.js';
import { getQueryInterface } from './util.js';
function traverseLinks(api, head) {
    const subject = new BehaviorSubject(head);
    const query = getQueryInterface(api);
    return subject.pipe(switchMap((account) => query.listNodes(account)), tap((node) => {
        nextTick(() => {
            node.isSome && node.value.next.isSome
                ? subject.next(node.unwrap().next.unwrap())
                : subject.complete();
        });
    }), toArray(), // toArray since we want to startSubject to be completed
    map((all) => all.map((o) => o.unwrap())));
}
/**
 * @name listNodes
 * @param {(PalletBagsListListBag | null)} bag A reference to a specific bag in the BagsList pallet.
 * @description Retrieves the list of nodes (accounts) contained in a specific bag within the BagsList pallet.
 */
export function listNodes(instanceId, api) {
    return memo(instanceId, (bag) => bag && bag.head.isSome
        ? traverseLinks(api, bag.head.unwrap())
        : of([]));
}
