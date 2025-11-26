"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.referendumsFinished = referendumsFinished;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name referendumsFinished
 * @description Retrieves information about finished referendums.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumsFinished();
 * console.log("Finished Referendums:", referendums);
 * ```
 */
function referendumsFinished(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.democracy.referendumIds().pipe((0, rxjs_1.switchMap)((ids) => api.query.democracy.referendumInfoOf.multi(ids)), (0, rxjs_1.map)((infos) => infos
        .map((o) => o.unwrapOr(null))
        .filter((info) => !!info && info.isFinished)
        .map((info) => info.asFinished))));
}
