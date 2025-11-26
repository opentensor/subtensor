"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.waitingInfo = waitingInfo;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const DEFAULT_FLAGS = { withController: true, withPrefs: true };
/**
 * @name waitingInfo
 * @param {StakingQueryFlags} flags? (Optional) Query flags to filter the staking data.
 * @description Staking candidates who are waiting to become validators.
 * @example
 * ```javascript
 * const { waiting, info } = await api.derive.staking.waitingInfo();
 * console.log(
 *   "Waiting Candidates:",
 *   waiting.map((acc) => acc.toString())
 * );
 * ```
 */
function waitingInfo(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (flags = DEFAULT_FLAGS) => (0, rxjs_1.combineLatest)([
        api.derive.staking.validators(),
        api.derive.staking.stashes()
    ]).pipe((0, rxjs_1.switchMap)(([{ nextElected }, stashes]) => {
        const elected = nextElected.map((a) => a.toString());
        const waiting = stashes.filter((v) => !elected.includes(v.toString()));
        return api.derive.staking.queryMulti(waiting, flags).pipe((0, rxjs_1.map)((info) => ({
            info,
            waiting
        })));
    })));
}
