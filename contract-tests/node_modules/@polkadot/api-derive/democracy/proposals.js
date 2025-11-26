import { combineLatest, map, of, switchMap } from 'rxjs';
import { isFunction, objectSpread } from '@polkadot/util';
import { memo } from '../util/index.js';
import { getImageHashBounded } from './util.js';
function isNewDepositors(depositors) {
    // Detect balance...
    return isFunction(depositors[1].mul);
}
function parse([proposals, images, optDepositors]) {
    return proposals
        .filter(([, , proposer], index) => !!(optDepositors[index]?.isSome) && !proposer.isEmpty)
        .map(([index, hash, proposer], proposalIndex) => {
        const depositors = optDepositors[proposalIndex].unwrap();
        return objectSpread({
            image: images[proposalIndex],
            imageHash: getImageHashBounded(hash),
            index,
            proposer
        }, isNewDepositors(depositors)
            ? { balance: depositors[1], seconds: depositors[0] }
            : { balance: depositors[0], seconds: depositors[1] });
    });
}
/**
 * @name proposals
 * @description Retrieves the list of active public proposals in the democracy module, along with their associated preimage data and deposit information.
 * @example
 * ```javascript
 * const proposals = await api.derive.democracy.proposals();
 * console.log("proposals:", proposals);
 * ```
 */
export function proposals(instanceId, api) {
    return memo(instanceId, () => isFunction(api.query.democracy?.publicProps)
        ? api.query.democracy.publicProps().pipe(switchMap((proposals) => proposals.length
            ? combineLatest([
                of(proposals),
                api.derive.democracy.preimages(proposals.map(([, hash]) => hash)),
                api.query.democracy.depositOf.multi(proposals.map(([index]) => index))
            ])
            : of([[], [], []])), map(parse))
        : of([]));
}
