"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.erasHistoric = erasHistoric;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
/**
 * @name erasHistoric
 * @param {boolean} withActive? (Optional) Whether to include the active era in the result.
 */
function erasHistoric(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (withActive) => (0, rxjs_1.combineLatest)([
        api.query.staking.activeEra(),
        api.consts.staking.historyDepth
            ? (0, rxjs_1.of)(api.consts.staking.historyDepth)
            : api.query.staking['historyDepth']()
    ]).pipe((0, rxjs_1.map)(([activeEraOpt, historyDepth]) => {
        const result = [];
        const max = historyDepth.toNumber();
        const activeEra = activeEraOpt.unwrapOrDefault().index;
        let lastEra = activeEra;
        while (lastEra.gte(util_1.BN_ZERO) && (result.length < max)) {
            if ((lastEra !== activeEra) || (withActive === true)) {
                result.push(api.registry.createType('EraIndex', lastEra));
            }
            lastEra = lastEra.sub(util_1.BN_ONE);
        }
        // go from oldest to newest
        return result.reverse();
    })));
}
