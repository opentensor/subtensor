"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stashes = stashes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function onBondedEvent(api) {
    let current = Date.now();
    return api.query.system.events().pipe((0, rxjs_1.map)((events) => {
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
    }), (0, rxjs_1.startWith)(current), (0, index_js_1.drr)({ skipTimeout: true }));
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
function stashes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => onBondedEvent(api).pipe((0, rxjs_1.switchMap)(() => api.query.staking.validators.keys()), (0, rxjs_1.map)((keys) => keys.map(({ args: [v] }) => v).filter((a) => a))));
}
