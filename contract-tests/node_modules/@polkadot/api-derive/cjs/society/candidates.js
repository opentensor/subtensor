"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.candidates = candidates;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function getPrev(api) {
    return api.query.society.candidates().pipe((0, rxjs_1.switchMap)((candidates) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(candidates),
        api.query.society['suspendedCandidates'].multi(candidates.map(({ who }) => who))
    ])), (0, rxjs_1.map)(([candidates, suspended]) => candidates.map(({ kind, value, who }, index) => ({
        accountId: who,
        isSuspended: suspended[index].isSome,
        kind,
        value
    }))));
}
function getCurr(api) {
    return api.query.society.candidates.entries().pipe((0, rxjs_1.map)((entries) => entries
        .filter(([, opt]) => opt.isSome)
        .map(([{ args: [accountId] }, opt]) => [accountId, opt.unwrap()])
        // FIXME We are missing the new fields from the candidate record
        .map(([accountId, { bid, kind }]) => ({
        accountId,
        isSuspended: false,
        kind,
        value: bid
    }))));
}
/**
 * @name candidate
 * @description Retrieves the list of candidates for the society module.
 * @example
 * ```javascript
 * const societyCandidates = await api.derive.society.candidates();
 * console.log(societyCandidates);
 * ```
 */
function candidates(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.society['suspendedCandidates'] && api.query.society.candidates.creator.meta.type.isPlain
        ? getPrev(api)
        : getCurr(api));
}
