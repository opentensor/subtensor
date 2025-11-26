import { combineLatest, map, of } from 'rxjs';
import { firstMemo, memo } from '../util/index.js';
import { erasHistoricApplyAccount } from './util.js';
export function _ownExposures(instanceId, api) {
    return memo(instanceId, (accountId, eras, _withActive, page) => {
        const emptyStakingExposure = api.registry.createType('Exposure');
        // The reason we don't explicitly make the actual types is for compatibility. If the chain doesn't have the noted type it will fail
        // on construction. Therefore we just make an empty option.
        const emptyOptionPage = api.registry.createType('Option<Null>');
        const emptyOptionMeta = api.registry.createType('Option<Null>');
        return eras.length
            ? combineLatest([
                // Backwards and forward compat for historical integrity when using `erasHistoricApplyAccount`
                api.query.staking.erasStakersClipped
                    ? combineLatest(eras.map((e) => api.query.staking.erasStakersClipped(e, accountId)))
                    : of(eras.map((_) => emptyStakingExposure)),
                api.query.staking.erasStakers
                    ? combineLatest(eras.map((e) => api.query.staking.erasStakers(e, accountId)))
                    : of(eras.map((_) => emptyStakingExposure)),
                api.query.staking.erasStakersPaged
                    ? combineLatest(eras.map((e) => api.query.staking.erasStakersPaged(e, accountId, page)))
                    : of(eras.map((_) => emptyOptionPage)),
                api.query.staking.erasStakersOverview
                    ? combineLatest(eras.map((e) => api.query.staking.erasStakersOverview(e, accountId)))
                    : of(eras.map((_) => emptyOptionMeta))
            ]).pipe(map(([clp, exp, paged, expMeta]) => eras.map((era, index) => ({ clipped: clp[index], era, exposure: exp[index], exposureMeta: expMeta[index], exposurePaged: paged[index] }))))
            : of([]);
    });
}
/**
 * @name ownExposure
 * @description Retrieves the staking exposure of a validator for a specific era, including their own stake.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param {EraIndex} era The staking era to query.
 * @param { u32 | AnyNumber } page? (Optional) The pagination index.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const exposure = await api.derive.staking.ownExposure(
 *   "11VR4pF6c7kfBhfmuwwjWY3FodeYBKWx7ix2rsRCU2q6hqJ",
 *   era
 * );
 * console.log(JSON.stringify(exposure));
 * ```
 */
export const ownExposure = /*#__PURE__*/ firstMemo((api, accountId, era, page) => api.derive.staking._ownExposures(accountId, [era], true, page || 0));
/**
 * @name ownExposures
 * @description Retrieves staking exposures for a validator across multiple historical eras.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const exposures = await api.derive.staking.ownExposures(
 *   ALICE,
 *   true
 * );
 * ```
 */
export const ownExposures = /*#__PURE__*/ erasHistoricApplyAccount('_ownExposures');
