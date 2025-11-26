"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ownSlashes = exports.ownSlash = void 0;
exports._ownSlashes = _ownSlashes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function _ownSlashes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId, eras, _withActive) => eras.length
        ? (0, rxjs_1.combineLatest)([
            (0, rxjs_1.combineLatest)(eras.map((e) => api.query.staking.validatorSlashInEra(e, accountId))),
            (0, rxjs_1.combineLatest)(eras.map((e) => api.query.staking.nominatorSlashInEra(e, accountId)))
        ]).pipe((0, rxjs_1.map)(([vals, noms]) => eras.map((era, index) => ({
            era,
            total: vals[index].isSome
                ? vals[index].unwrap()[1]
                : noms[index].unwrapOrDefault()
        }))))
        : (0, rxjs_1.of)([]));
}
/**
 * @name ownSlash
 * @description Retrieves the slashes applied to a specific account in a given era.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param {EraIndex} era The staking era to query.
 * @example
 * ```javascript
 * const era = api.createType("EraIndex", 1000);
 * const slashedAmount = await api.derive.staking.ownSlash(
 *   ALICE,
 *   era
 * );
 * console.log(`Era: ${slashedAmount.era}, total ${slashedAmount.total}`);
 * ```
 */
exports.ownSlash = (0, index_js_1.firstMemo)((api, accountId, era) => api.derive.staking._ownSlashes(accountId, [era], true));
/**
 * @name ownSlashes
 * @description Retrieves the slashes for a specific account across all historic eras.
 * @param { Uint8Array | string } accountId The validator stash account.
 * @param { boolean } withActive Whether to include the active era.
 * @example
 * ```javascript
 * const slashes = await api.derive.staking.ownSlashes(
 *   ALICE,
 *   true
 * );
 * console.log(slashes);
 * ```
 */
exports.ownSlashes = (0, util_js_1.erasHistoricApplyAccount)('_ownSlashes');
