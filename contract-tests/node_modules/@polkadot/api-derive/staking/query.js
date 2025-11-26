import { combineLatest, map, of, switchMap } from 'rxjs';
import { firstMemo, memo } from '../util/index.js';
function rewardDestinationCompat(rewardDestination) {
    // We ensure the type is an Option by checking if isSome is a boolean. When isSome doesn't exist it will always return undefined.
    return typeof rewardDestination.isSome === 'boolean'
        ? rewardDestination.unwrapOr(null)
        : rewardDestination;
}
function filterClaimedRewards(api, cl) {
    return api.registry.createType('Vec<u32>', cl.filter((c) => c !== -1));
}
function filterRewards(stashIds, eras, claimedRewards, stakersOverview) {
    const claimedData = {};
    const overviewData = {};
    const ids = stashIds.map((i) => i.toString());
    claimedRewards.forEach(([keys, rewards]) => {
        const id = keys.args[1].toString();
        const era = keys.args[0].toNumber();
        if (ids.includes(id)) {
            if (claimedData[id]) {
                claimedData[id].set(era, rewards.toArray());
            }
            else {
                claimedData[id] = new Map();
                claimedData[id].set(era, rewards.toArray());
            }
        }
    });
    stakersOverview.forEach(([keys, overview]) => {
        const id = keys.args[1].toString();
        const era = keys.args[0].toNumber();
        if (ids.includes(id) && overview.isSome) {
            if (overviewData[id]) {
                overviewData[id].set(era, overview.unwrap().pageCount);
            }
            else {
                overviewData[id] = new Map();
                overviewData[id].set(era, overview.unwrap().pageCount);
            }
        }
    });
    return stashIds.map((id) => {
        const rewardsPerEra = claimedData[id.toString()];
        const overviewPerEra = overviewData[id.toString()];
        return eras.map((era) => {
            if (rewardsPerEra && rewardsPerEra.has(era) && overviewPerEra && overviewPerEra.has(era)) {
                const rewards = rewardsPerEra.get(era);
                const pageCount = overviewPerEra.get(era);
                return rewards.length === pageCount.toNumber()
                    ? era
                    : -1;
            }
            return -1;
        });
    });
}
function parseDetails(api, stashId, controllerIdOpt, nominatorsOpt, rewardDestinationOpts, validatorPrefs, exposure, stakingLedgerOpt, exposureMeta, claimedRewards, exposureEraStakers) {
    return {
        accountId: stashId,
        claimedRewardsEras: filterClaimedRewards(api, claimedRewards),
        controllerId: controllerIdOpt?.unwrapOr(null) || null,
        exposureEraStakers,
        exposureMeta,
        exposurePaged: exposure,
        nominators: nominatorsOpt.isSome
            ? nominatorsOpt.unwrap().targets
            : [],
        rewardDestination: rewardDestinationCompat(rewardDestinationOpts),
        stakingLedger: stakingLedgerOpt.unwrapOrDefault(),
        stashId,
        validatorPrefs
    };
}
function getLedgers(api, optIds, { withLedger = false }) {
    const ids = optIds
        .filter((o) => withLedger && !!o && o.isSome)
        .map((o) => o.unwrap());
    const emptyLed = api.registry.createType('Option<StakingLedger>');
    return (ids.length
        ? combineLatest(ids.map((s) => api.query.staking.ledger(s)))
        : of([])).pipe(map((optLedgers) => {
        let offset = -1;
        return optIds.map((o) => o && o.isSome
            ? optLedgers[++offset] || emptyLed
            : emptyLed);
    }));
}
function getStashInfo(api, stashIds, activeEra, { withClaimedRewardsEras, withController, withDestination, withExposure, withExposureErasStakersLegacy, withExposureMeta, withLedger, withNominations, withPrefs }, page) {
    const emptyNoms = api.registry.createType('Option<Nominations>');
    const emptyRewa = api.registry.createType('RewardDestination');
    const emptyExpoEraStakers = api.registry.createType('Exposure');
    const emptyPrefs = api.registry.createType('ValidatorPrefs');
    // The reason we don't explicitly make the actual types is for compatibility. If the chain doesn't have the noted type it will fail
    // on construction. Therefore we just make an empty option.
    const emptyExpo = api.registry.createType('Option<Null>');
    const emptyExpoMeta = api.registry.createType('Option<Null>');
    const emptyClaimedRewards = [-1];
    const depth = Number(api.consts.staking.historyDepth.toNumber());
    const eras = new Array(depth).fill(0).map((_, idx) => {
        if (idx === 0) {
            return activeEra.toNumber() - 1;
        }
        return activeEra.toNumber() - idx - 1;
    });
    return combineLatest([
        withController || withLedger
            ? combineLatest(stashIds.map((s) => api.query.staking.bonded(s)))
            : of(stashIds.map(() => null)),
        withNominations
            ? combineLatest(stashIds.map((s) => api.query.staking.nominators(s)))
            : of(stashIds.map(() => emptyNoms)),
        withDestination
            ? combineLatest(stashIds.map((s) => api.query.staking.payee(s)))
            : of(stashIds.map(() => emptyRewa)),
        withPrefs
            ? combineLatest(stashIds.map((s) => api.query.staking.validators(s)))
            : of(stashIds.map(() => emptyPrefs)),
        withExposure && api.query.staking.erasStakersPaged
            ? combineLatest(stashIds.map((s) => api.query.staking.erasStakersPaged(activeEra, s, page)))
            : of(stashIds.map(() => emptyExpo)),
        withExposureMeta && api.query.staking.erasStakersOverview
            ? combineLatest(stashIds.map((s) => api.query.staking.erasStakersOverview(activeEra, s)))
            : of(stashIds.map(() => emptyExpoMeta)),
        withClaimedRewardsEras && api.query.staking.claimedRewards
            ? combineLatest([
                api.query.staking.claimedRewards.entries(),
                api.query.staking.erasStakersOverview.entries()
            ]).pipe(map(([rewardsStorageVec, overviewStorageVec]) => filterRewards(stashIds, eras, rewardsStorageVec, overviewStorageVec)))
            : of(stashIds.map(() => emptyClaimedRewards)),
        withExposureErasStakersLegacy && api.query.staking.erasStakers
            ? combineLatest(stashIds.map((s) => api.query.staking.erasStakers(activeEra, s)))
            : of(stashIds.map(() => emptyExpoEraStakers))
    ]);
}
function getBatch(api, activeEra, stashIds, flags, page) {
    return getStashInfo(api, stashIds, activeEra, flags, page).pipe(switchMap(([controllerIdOpt, nominatorsOpt, rewardDestination, validatorPrefs, exposure, exposureMeta, claimedRewardsEras, exposureEraStakers]) => getLedgers(api, controllerIdOpt, flags).pipe(map((stakingLedgerOpts) => stashIds.map((stashId, index) => parseDetails(api, stashId, controllerIdOpt[index], nominatorsOpt[index], rewardDestination[index], validatorPrefs[index], exposure[index], stakingLedgerOpts[index], exposureMeta[index], claimedRewardsEras[index], exposureEraStakers[index]))))));
}
/**
 * @name query
 * @param { Uint8Array | string } accountId The stash account to query.
 * @param { StakingQueryFlags } flags Flags to customize the query.
 * @param { u32 } page (Optional) pagination parameter.
 * @description Retrieves staking details for a given stash account.
 * @example
 * ```javascript
 * const stakingInfo = await api.derive.staking.query(
 *   ALICE,
 *   {}
 * );
 * ```
 */
export const query = /*#__PURE__*/ firstMemo((api, accountId, flags, page) => api.derive.staking.queryMulti([accountId], flags, page));
/**
 * @name queryMulti
 * @param { (Uint8Array | string)[] } accountIds List of stash accounts to query.
 * @param { StakingQueryFlags } flags Flags to customize the query.
 * @param { u32 } page (Optional) pagination parameter.
 * @description Retrieves staking details for multiple stash accounts.
 * @example
 * ```javascript
 * const stakingInfos = await api.derive.staking.queryMulti([stashId1, stashId2], {});
 * ```
 */
export function queryMulti(instanceId, api) {
    return memo(instanceId, (accountIds, flags, page) => api.derive.session.indexes().pipe(switchMap(({ activeEra }) => {
        const stashIds = accountIds.map((a) => api.registry.createType('AccountId', a));
        const p = page || 0;
        return stashIds.length
            ? getBatch(api, activeEra, stashIds, flags, p)
            : of([]);
    })));
}
