"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.account = void 0;
exports.accounts = accounts;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const QUERY_OPTS = {
    withDestination: true,
    withLedger: true,
    withNominations: true,
    withPrefs: true
};
function groupByEra(list) {
    return list.reduce((map, { era, value }) => {
        const key = era.toString();
        map[key] = (map[key] || util_1.BN_ZERO).add(value.unwrap());
        return map;
    }, {});
}
function calculateUnlocking(api, stakingLedger, sessionInfo) {
    const results = Object
        .entries(groupByEra((stakingLedger?.unlocking || []).filter(({ era }) => era.unwrap().gt(sessionInfo.activeEra))))
        .map(([eraString, value]) => ({
        remainingEras: new util_1.BN(eraString).isub(sessionInfo.activeEra),
        value: api.registry.createType('Balance', value)
    }));
    return results.length
        ? results
        : undefined;
}
function redeemableSum(api, stakingLedger, sessionInfo) {
    return api.registry.createType('Balance', (stakingLedger?.unlocking || []).reduce((total, { era, value }) => {
        // aligns with https://github.com/paritytech/substrate/blob/fdfdc73f9e64dc47934b72eb9af3e1989e4ba699/frame/staking/src/pallet/mod.rs#L973-L975
        // (ensure currentEra >= era passed, as per https://github.com/paritytech/substrate/blob/fdfdc73f9e64dc47934b72eb9af3e1989e4ba699/frame/staking/src/lib.rs#L477-L494)
        // NOTE: Previously we used activeEra >= era, which is incorrect for the last session
        return era.unwrap().gt(sessionInfo.currentEra)
            ? total
            : total.iadd(value.unwrap());
    }, new util_1.BN(0)));
}
function parseResult(api, sessionInfo, keys, query) {
    return (0, util_1.objectSpread)({}, keys, query, {
        redeemable: redeemableSum(api, query.stakingLedger, sessionInfo),
        unlocking: calculateUnlocking(api, query.stakingLedger, sessionInfo)
    });
}
/**
 * @name accounts
 * @param {(Uint8Array | string)[]} accountIds List of account stashes
 * @param {StakingQueryFlags} opts optional filtering flag
 * @description From a list of stashes, fill in all the relevant staking details
 * @example
 * ```javascript
 * const accounts = await api.derive.staking.accounts([
 *  "149B17nn7zVL4SkLSNmANupEkGexUBAxVrdk4bbWFZYibkFc",
 * ]);
 * console.log("First account staking info:", accounts[0]);
 * ```
 */
function accounts(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountIds, opts = QUERY_OPTS) => api.derive.session.info().pipe((0, rxjs_1.switchMap)((sessionInfo) => (0, rxjs_1.combineLatest)([
        api.derive.staking.keysMulti(accountIds),
        api.derive.staking.queryMulti(accountIds, opts)
    ]).pipe((0, rxjs_1.map)(([keys, queries]) => queries.map((q, index) => parseResult(api, sessionInfo, keys[index], q)))))));
}
/**
 * @name account
 * @param {(Uint8Array | string)} accountId AccountId of the stash.
 * @param {StakingQueryFlags} opts (Optional) filtering flag.
 * @description From a stash, retrieve the controllerId and fill in all the relevant staking details.
 * @example
 * ```javascript
 * const accountStakingData = await api.derive.staking.account(
 *   "149B17nn7zVL4SkLSNmANupEkGexUBAxVrdk4bbWFZYibkFc"
 * );
 * console.log(accountStakingData);
 * ```
 */
exports.account = (0, index_js_1.firstMemo)((api, accountId, opts) => api.derive.staking.accounts([accountId], opts));
