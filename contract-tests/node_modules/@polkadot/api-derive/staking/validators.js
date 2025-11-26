import { combineLatest, map, of, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
/**
 * @name nextElected
 * @description Retrieves the list of accounts that are set to be the next elected validators in the staking system. It provides a preview of who will be validators in the next staking era.
 * @example
 * ```javascript
 * const nextElected = await api.derive.staking.nextElected();
 * console.log(
 *   "Next Elected Validators:",
 *   nextElected.map((acc) => acc.toString())
 * );
 * ```
 */
export function nextElected(instanceId, api) {
    return memo(instanceId, () => 
    // Compatibility for future generation changes in staking.
    api.query.staking.erasStakersOverview
        ? api.derive.session.indexes().pipe(
        // only populate for next era in the last session, so track both here - entries are not
        // subscriptions, so we need a trigger - currentIndex acts as that trigger to refresh
        switchMap(({ currentEra }) => api.query.staking.erasStakersOverview.keys(currentEra)), 
        // Dedupe any duplicates
        map((keys) => [...new Set(keys.map(({ args: [, accountId] }) => accountId.toString()))].map((a) => api.registry.createType('AccountId', a))))
        : api.query.staking.erasStakers
            ? api.derive.session.indexes().pipe(
            // only populate for next era in the last session, so track both here - entries are not
            // subscriptions, so we need a trigger - currentIndex acts as that trigger to refresh
            switchMap(({ currentEra }) => api.query.staking.erasStakers.keys(currentEra)), 
            // Dedupe any duplicates
            map((keys) => [...new Set(keys.map(({ args: [, accountId] }) => accountId.toString()))].map((a) => api.registry.createType('AccountId', a))))
            : api.query.staking['currentElected']());
}
/**
 * @name validators
 * @description Retrieve latest list of validators.
 * @example
 * ```javascript
 * const { validators, nextElected } = await api.derive.staking.validators();
 * console.log(
 *   "Current Validators:",
 *   validators.map((v) => v.toString())
 * );
 * console.log(
 *   "Next Elected Validators:",
 *   nextElected.map((v) => v.toString())
 * );
 * ```
 */
export function validators(instanceId, api) {
    return memo(instanceId, () => 
    // Sadly the node-template is (for some obscure reason) not comprehensive, so while the derive works
    // in all actual real-world deployed chains, it does create some confusion for limited template chains
    combineLatest([
        api.query.session
            ? api.query.session.validators()
            : of([]),
        api.query.staking
            ? api.derive.staking.nextElected()
            : of([])
    ]).pipe(map(([validators, nextElected]) => ({
        nextElected: nextElected.length
            ? nextElected
            : validators,
        validators
    }))));
}
