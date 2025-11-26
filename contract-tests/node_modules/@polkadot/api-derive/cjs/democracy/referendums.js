"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.referendums = referendums;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
/**
 * @name referendums
 * @description Retrieves information about all active referendums, including their details and associated votes.
 * @example
 * ```javascript
 * const referendums = await api.derive.democracy.referendums();
 * ```
 */
function referendums(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.derive.democracy.referendumsActive().pipe((0, rxjs_1.switchMap)((referendums) => referendums.length
        ? (0, rxjs_1.combineLatest)([
            (0, rxjs_1.of)(referendums),
            api.derive.democracy._referendumsVotes(referendums)
        ])
        : (0, rxjs_1.of)([[], []])), (0, rxjs_1.map)(([referendums, votes]) => referendums.map((referendum, index) => (0, util_1.objectSpread)({}, referendum, votes[index])))));
}
