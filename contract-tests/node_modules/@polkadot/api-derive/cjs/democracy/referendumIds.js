"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.referendumIds = referendumIds;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name referendumIds
 * @description Retrieves an array of active referendum IDs.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendumIds();
 * ```
 */
function referendumIds(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.democracy?.lowestUnbaked
        ? api.queryMulti([
            api.query.democracy.lowestUnbaked,
            api.query.democracy.referendumCount
        ]).pipe((0, rxjs_1.map)(([first, total]) => total.gt(first)
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            ? [...Array(total.sub(first).toNumber())].map((_, i) => first.addn(i))
            : []))
        : (0, rxjs_1.of)([]));
}
