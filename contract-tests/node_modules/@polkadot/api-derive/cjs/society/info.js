"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.info = info;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name info
 * @description Get the overall info for a society.
 * @example
 * ```javascript
 * const societyInfo = await api.derive.society.candidates();
 * console.log(societyInfo);
 * ```
 */
function info(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => (0, rxjs_1.combineLatest)([
        api.query.society.bids(),
        api.query.society['defender']
            ? api.query.society['defender']()
            : (0, rxjs_1.of)(undefined),
        api.query.society.founder(),
        api.query.society.head(),
        api.query.society['maxMembers']
            ? api.query.society['maxMembers']()
            : (0, rxjs_1.of)(undefined),
        api.query.society.pot()
    ]).pipe((0, rxjs_1.map)(([bids, defender, founder, head, maxMembers, pot]) => ({
        bids,
        defender: defender?.unwrapOr(undefined),
        founder: founder.unwrapOr(undefined),
        hasDefender: (defender?.isSome && head.isSome && !head.eq(defender)) || false,
        head: head.unwrapOr(undefined),
        maxMembers,
        pot
    }))));
}
