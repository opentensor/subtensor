import { catchError, combineLatest, map, of, switchMap } from 'rxjs';
import { Enum } from '@polkadot/types';
import { isFunction, objectSpread, stringToHex } from '@polkadot/util';
import { memo } from '../util/index.js';
import { getImageHashBounded } from './util.js';
const DEMOCRACY_ID = stringToHex('democrac');
function isMaybeHashedOrBounded(call) {
    // check for enum
    return call instanceof Enum;
}
function isBounded(call) {
    // check for type
    return call.isInline || call.isLegacy || call.isLookup;
}
function queryQueue(api) {
    return api.query.democracy['dispatchQueue']().pipe(switchMap((dispatches) => combineLatest([
        of(dispatches),
        api.derive.democracy.preimages(dispatches.map(([, hash]) => hash))
    ])), map(([dispatches, images]) => dispatches.map(([at, imageHash, index], dispatchIndex) => ({
        at,
        image: images[dispatchIndex],
        imageHash: getImageHashBounded(imageHash),
        index
    }))));
}
function schedulerEntries(api) {
    // We don't get entries, but rather we get the keys (triggered via finished referendums) and
    // the subscribe to those keys - this means we pickup when the schedulers actually executes
    // at a block, the entry for that block will become empty
    return api.derive.democracy.referendumsFinished().pipe(switchMap(() => api.query.scheduler.agenda.keys()), switchMap((keys) => {
        const blockNumbers = keys.map(({ args: [blockNumber] }) => blockNumber);
        return blockNumbers.length
            ? combineLatest([
                of(blockNumbers),
                // this should simply be api.query.scheduler.agenda.multi,
                // however we have had cases on Darwinia where the indices have moved around after an
                // upgrade, which results in invalid on-chain data
                api.query.scheduler.agenda.multi(blockNumbers).pipe(catchError(() => of(blockNumbers.map(() => []))))
            ])
            : of([[], []]);
    }));
}
function queryScheduler(api) {
    return schedulerEntries(api).pipe(switchMap(([blockNumbers, agendas]) => {
        const result = [];
        blockNumbers.forEach((at, index) => {
            (agendas[index] || []).filter((o) => o.isSome).forEach((o) => {
                const scheduled = o.unwrap();
                if (scheduled.maybeId.isSome) {
                    const id = scheduled.maybeId.unwrap().toHex();
                    if (id.startsWith(DEMOCRACY_ID)) {
                        const imageHash = isMaybeHashedOrBounded(scheduled.call)
                            ? isBounded(scheduled.call)
                                ? getImageHashBounded(scheduled.call)
                                : scheduled.call.isHash
                                    ? scheduled.call.asHash.toHex()
                                    : scheduled.call.asValue.args[0].toHex()
                            : scheduled.call.args[0].toHex();
                        result.push({ at, imageHash, index: api.registry.createType('(u64, ReferendumIndex)', id)[1] });
                    }
                }
            });
        });
        return combineLatest([
            of(result),
            result.length
                ? api.derive.democracy.preimages(result.map(({ imageHash }) => imageHash))
                : of([])
        ]);
    }), map(([infos, images]) => infos.map((info, index) => objectSpread({ image: images[index] }, info))));
}
/**
 * @name dispatchQueue
 * @description Retrieves the list of scheduled or pending dispatches in the governance system.
 * @example
 * ```javascript
 * const queue = await api.derive.democracy.dispatchQueue();
 * console.log("Dispatch Queue:", queue);
 * ```
 */
export function dispatchQueue(instanceId, api) {
    return memo(instanceId, () => isFunction(api.query.scheduler?.agenda)
        ? queryScheduler(api)
        : api.query.democracy['dispatchQueue']
            ? queryQueue(api)
            : of([]));
}
