"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ownExposures = exports.ownExposure = void 0;
exports._ownExposures = _ownExposures;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function _ownExposures(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId, eras, _withActive, page) => {
        const emptyStakingExposure = api.registry.createType('Exposure');
        // The reason we don't explicitly make the actual types is for compatibility. If the chain doesn't have the noted type it will fail
        // on construction. Therefore we just make an empty option.
        const emptyOptionPage = api.registry.createType('Option<Null>');
        const emptyOptionMeta = api.registry.createType('Option<Null>');
        return eras.length
            ? (0, rxjs_1.combineLatest)([
                // Backwards and forward compat for historical integrity when using `erasHistoricApplyAccount`
                api.query.staking.erasStakersClipped
                    ? (0, rxjs_1.combineLatest)(eras.map((e) => api.query.staking.erasStakersClipped(e, accountId)))
                    : (0, rxjs_1.of)(eras.map((_) => emptyStakingExposure)),
                api.query.staking.erasStakers
                    ? (0, rxjs_1.combineLatest)(eras.map((e) => api.query.staking.erasStakers(e, accountId)))
                    : (0, rxjs_1.of)(eras.map((_) => emptyStakingExposure)),
                api.query.staking.erasStakersPaged
                    ? (0, rxjs_1.combineLatest)(eras.map((e) => api.query.staking.erasStakersPaged(e, accountId, page)))
                    : (0, rxjs_1.of)(eras.map((_) => emptyOptionPage)),
                api.query.staking.erasStakersOverview
                    ? (0, rxjs_1.combineLatest)(eras.map((e) => api.query.staking.erasStakersOverview(e, accountId)))
                    : (0, rxjs_1.of)(eras.map((_) => emptyOptionMeta))
            ]).pipe((0, rxjs_1.map)(([clp, exp, paged, expMeta]) => eras.map((era, index) => ({ clipped: clp[index], era, exposure: exp[index], exposureMeta: expMeta[index], exposurePaged: paged[index] }))))
            : (0, rxjs_1.of)([]);
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
exports.ownExposure = (0, index_js_1.firstMemo)((api, accountId, era, page) => api.derive.staking._ownExposures(accountId, [era], true, page || 0));
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
exports.ownExposures = (0, util_js_1.erasHistoricApplyAccount)('_ownExposures');
