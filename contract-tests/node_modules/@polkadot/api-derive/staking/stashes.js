import { map, startWith, switchMap } from 'rxjs';
import { drr, memo } from '../util/index.js';
function onBondedEvent(api) {
    let current = Date.now();
    return api.query.system.events().pipe(map((events) => {
        current = events.filter(({ event, phase }) => {
            try {
                return phase.isApplyExtrinsic &&
                    event.section === 'staking' &&
                    event.method === 'Bonded';
            }
            catch {
                return false;
            }
        })
            ? Date.now()
            : current;
        return current;
    }), startWith(current), drr({ skipTimeout: true }));
}
/**
 * @name stashes
 * @description Retrieve the list of all validator stashes.
 * @example
 * ```javascript
 * const stashes = await api.derive.staking.stashes();
 * console.log(
 *   "Validator Stashes:",
 *   stashes.map((s) => s.toString())
 * );
 * ```
 */
export function stashes(instanceId, api) {
    return memo(instanceId, () => onBondedEvent(api).pipe(switchMap(() => api.query.staking.validators.keys()), map((keys) => keys.map(({ args: [v] }) => v).filter((a) => a))));
}
