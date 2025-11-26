"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.referendumsActive = referendumsActive;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name referendumsActive
 * @description Retrieves information about active referendums.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumsActive();
 * console.log("Active Referendums:", referendums);
 * ```
 */
function referendumsActive(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.democracy.referendumIds().pipe((0, rxjs_1.switchMap)((ids) => ids.length
        ? api.derive.democracy.referendumsInfo(ids)
        : (0, rxjs_1.of)([]))));
}
