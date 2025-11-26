import { combineLatest, map, of, switchMap } from 'rxjs';
import { BN_BILLION, BN_ZERO, objectSpread } from '@polkadot/util';
import { firstMemo, memo } from '../util/index.js';
function extractCompatRewards(claimedRewardsEras, ledger) {
    const l = ledger
        ? (ledger.legacyClaimedRewards ||
            ledger.claimedRewards)?.toArray()
        : [];
    return (claimedRewardsEras.toArray() || []).concat(l);
}
function parseRewards(api, stashId, [erasPoints, erasPrefs, erasRewards], exposures, claimedRewardsEras) {
    return exposures.map(({ era, isEmpty, isValidator, nominating, validators: eraValidators }) => {
        const { eraPoints, validators: allValPoints } = erasPoints.find((p) => p.era.eq(era)) || { eraPoints: BN_ZERO, validators: {} };
        const { eraReward } = erasRewards.find((r) => r.era.eq(era)) || { eraReward: api.registry.createType('Balance') };
        const { validators: allValPrefs } = erasPrefs.find((p) => p.era.eq(era)) || { validators: {} };
        const validators = {};
        const stakerId = stashId.toString();
        Object.entries(eraValidators).forEach(([validatorId, exposure]) => {
            const valPoints = allValPoints[validatorId] || BN_ZERO;
            const valComm = allValPrefs[validatorId]?.commission.unwrap() || BN_ZERO;
            const expTotal = exposure.total
                ? exposure.total?.unwrap()
                : exposure.pageTotal
                    ? exposure.pageTotal?.unwrap()
                    : BN_ZERO;
            let avail = BN_ZERO;
            let value;
            if (!(expTotal.isZero() || valPoints.isZero() || eraPoints.isZero())) {
                avail = eraReward.mul(valPoints).div(eraPoints);
                const valCut = valComm.mul(avail).div(BN_BILLION);
                let staked;
                if (validatorId === stakerId) {
                    if (exposure.own) {
                        staked = exposure.own.unwrap();
                    }
                    else {
                        const expAccount = exposure.others.find(({ who }) => who.eq(validatorId));
                        staked = expAccount
                            ? expAccount.value.unwrap()
                            : BN_ZERO;
                    }
                }
                else {
                    const stakerExp = exposure.others.find(({ who }) => who.eq(stakerId));
                    staked = stakerExp
                        ? stakerExp.value.unwrap()
                        : BN_ZERO;
                }
                value = avail.sub(valCut).imul(staked).div(expTotal).iadd(validatorId === stakerId ? valCut : BN_ZERO);
            }
            validators[validatorId] = {
                total: api.registry.createType('Balance', avail),
                value: api.registry.createType('Balance', value)
            };
        });
        return {
            era,
            eraReward,
            // This might not always be accurate as you need validator account information in order to see if the rewards have been claimed.
            // This is possibly adjusted in `filterRewards` when need be.
            isClaimed: claimedRewardsEras.some((c) => c.eq(era)),
            isEmpty,
            isValidator,
            nominating,
            validators
        };
    });
}
function allUniqValidators(rewards) {
    return rewards.reduce(([all, perStash], rewards) => {
        const uniq = [];
        perStash.push(uniq);
        rewards.forEach(({ validators }) => Object.keys(validators).forEach((validatorId) => {
            if (!uniq.includes(validatorId)) {
                uniq.push(validatorId);
                if (!all.includes(validatorId)) {
                    all.push(validatorId);
                }
            }
        }));
        return [all, perStash];
    }, [[], []]);
}
function removeClaimed(validators, queryValidators, reward, claimedRewardsEras) {
    const rm = [];
    Object.keys(reward.validators).forEach((validatorId) => {
        const index = validators.indexOf(validatorId);
        if (index !== -1) {
            const valLedger = queryValidators[index].stakingLedger;
            if (extractCompatRewards(claimedRewardsEras, valLedger).some((e) => reward.era?.eq(e))) {
                rm.push(validatorId);
            }
        }
    });
    rm.forEach((validatorId) => {
        delete reward.validators[validatorId];
    });
}
function filterRewards(eras, valInfo, { claimedRewardsEras, rewards, stakingLedger }) {
    const filter = eras.filter((e) => !extractCompatRewards(claimedRewardsEras, stakingLedger).some((s) => s?.eq(e)));
    const validators = valInfo.map(([v]) => v);
    const queryValidators = valInfo.map(([, q]) => q);
    return rewards
        .filter(({ isEmpty }) => !isEmpty)
        .filter((reward) => {
        if (!filter.some((e) => reward.era.eq(e))) {
            return false;
        }
        removeClaimed(validators, queryValidators, reward, claimedRewardsEras);
        return true;
    })
        .filter(({ validators }) => Object.keys(validators).length !== 0)
        .map((reward) => {
        let isClaimed = reward.isClaimed;
        const valKeys = Object.keys(reward.validators);
        if (!reward.isClaimed && valKeys.length) {
            for (const key of valKeys) {
                const info = queryValidators.find((i) => i.accountId.toString() === key);
                if (info) {
                    isClaimed = info.claimedRewardsEras?.toArray().some((era) => era.eq(reward.era));
                    break;
                }
            }
        }
        return objectSpread({}, reward, {
            isClaimed,
            nominators: reward.nominating.filter((n) => reward.validators[n.validatorId])
        });
    });
}
export function _stakerRewardsEras(instanceId, api) {
    return memo(instanceId, (eras, withActive = false) => combineLatest([
        api.derive.staking._erasPoints(eras, withActive),
        api.derive.staking._erasPrefs(eras, withActive),
        api.derive.staking._erasRewards(eras, withActive)
    ]));
}
export function _stakerRewards(instanceId, api) {
    return memo(instanceId, (accountIds, eras, withActive = false) => {
        // Ensures that when number or string types are passed in they are sanitized
        // Ref: https://github.com/polkadot-js/api/issues/5910
        const sanitizedEras = eras.map((e) => typeof e === 'number' || typeof e === 'string' ? api.registry.createType('u32', e) : e);
        return combineLatest([
            api.derive.staking.queryMulti(accountIds, { withClaimedRewardsEras: true, withLedger: true }),
            api.derive.staking._stakerExposures(accountIds, sanitizedEras, withActive),
            api.derive.staking._stakerRewardsEras(sanitizedEras, withActive)
        ]).pipe(switchMap(([queries, exposures, erasResult]) => {
            const allRewards = queries.map(({ claimedRewardsEras, stakingLedger, stashId }, index) => (!stashId || (!stakingLedger && !claimedRewardsEras))
                ? []
                : parseRewards(api, stashId, erasResult, exposures[index], claimedRewardsEras));
            if (withActive) {
                return of(allRewards);
            }
            const [allValidators, stashValidators] = allUniqValidators(allRewards);
            return api.derive.staking.queryMulti(allValidators, { withClaimedRewardsEras: true, withLedger: true }).pipe(map((queriedVals) => queries.map(({ claimedRewardsEras, stakingLedger }, index) => filterRewards(eras, stashValidators[index]
                .map((validatorId) => [
                validatorId,
                queriedVals.find((q) => q.accountId.eq(validatorId))
            ])
                .filter((v) => !!v[1]), {
                claimedRewardsEras,
                rewards: allRewards[index],
                stakingLedger
            }))));
        }));
    });
}
/**
 * @name stakerRewards
 * @description Staking rewards history for a given staker.
 * @param { Uint8Array | string } accountId The stakers AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.stakerRewards(
 *   ALICE, //Alice accountId
 *   false
 * );
 * ```
 */
export const stakerRewards = /*#__PURE__*/ firstMemo((api, accountId, withActive) => api.derive.staking.erasHistoric(withActive).pipe(switchMap((eras) => api.derive.staking._stakerRewards([accountId], eras, withActive))));
/**
 * @name stakerRewardsMultiEras
 * @description Staking rewards for multiple stakers over specific eras.
 * @param { Uint8Array | string } accountIds List of stakers identified by their AccountId.
 * @param { EraIndex[] } eras Eras for which to retrieve the data.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.stakerRewardsMultiEras(
 *   [ALICE, BOB, CHARLIER], //accountIds
 *   [100,101]  //eras
 * );
 * ```
 */
export function stakerRewardsMultiEras(instanceId, api) {
    return memo(instanceId, (accountIds, eras) => accountIds.length && eras.length
        ? api.derive.staking._stakerRewards(accountIds, eras, false)
        : of([]));
}
/**
 * @name stakerRewardsMulti
 * @description Staking rewards for multiple stakers.
 * @param { Uint8Array | string } accountIds List of stakers identified by their AccountId.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const rewards = await api.derive.staking.stakerRewardsMulti(
 *   [ALICE, BOB, CHARLIER], //accountIds
 *   true
 * );
 * ```
 */
export function stakerRewardsMulti(instanceId, api) {
    return memo(instanceId, (accountIds, withActive = false) => api.derive.staking.erasHistoric(withActive).pipe(switchMap((eras) => api.derive.staking.stakerRewardsMultiEras(accountIds, eras))));
}
