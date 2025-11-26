"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports._members = _members;
exports.members = members;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function _membersPrev(api, accountIds) {
    return (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(accountIds),
        api.query.society.payouts.multi(accountIds),
        api.query.society['strikes'].multi(accountIds),
        api.query.society.defenderVotes.multi(accountIds),
        api.query.society.suspendedMembers.multi(accountIds),
        api.query.society['vouching'].multi(accountIds)
    ]).pipe((0, rxjs_1.map)(([accountIds, payouts, strikes, defenderVotes, suspended, vouching]) => accountIds.map((accountId, index) => ({
        accountId,
        isDefenderVoter: defenderVotes[index].isSome,
        isSuspended: suspended[index].isTrue,
        payouts: payouts[index],
        strikes: strikes[index],
        vote: defenderVotes[index].unwrapOr(undefined),
        vouching: vouching[index].unwrapOr(undefined)
    }))));
}
function _membersCurr(api, accountIds) {
    return (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(accountIds),
        api.query.society.members.multi(accountIds),
        api.query.society.payouts.multi(accountIds),
        api.query.society.challengeRoundCount().pipe((0, rxjs_1.switchMap)((round) => api.query.society.defenderVotes.multi(accountIds.map((accountId) => [round, accountId])))),
        api.query.society.suspendedMembers.multi(accountIds)
    ]).pipe((0, rxjs_1.map)(([accountIds, members, payouts, defenderVotes, suspendedMembers]) => accountIds
        .map((accountId, index) => members[index].isSome
        ? {
            accountId,
            isDefenderVoter: defenderVotes[index].isSome,
            isSuspended: suspendedMembers[index].isSome,
            member: members[index].unwrap(),
            payouts: payouts[index].payouts
        }
        : null)
        .filter((m) => !!m)
        .map(({ accountId, isDefenderVoter, isSuspended, member, payouts }) => ({
        accountId,
        isDefenderVoter,
        isSuspended,
        payouts,
        strikes: member.strikes,
        vouching: member.vouching.unwrapOr(undefined)
    }))));
}
function _members(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountIds) => api.query.society.members.creator.meta.type.isMap
        ? _membersCurr(api, accountIds)
        : _membersPrev(api, accountIds));
}
/**
 * @name members
 * @description Get the society members.
 * @example
 * ```javascript
 * const members = await api.derive.society.members();
 * console.log(members);
 * ```
 */
function members(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => api.query.society.members.creator.meta.type.isMap
        ? api.query.society.members.keys().pipe((0, rxjs_1.switchMap)((keys) => api.derive.society._members(keys.map(({ args: [accountId] }) => accountId))))
        : api.query.society.members().pipe((0, rxjs_1.switchMap)((members) => api.derive.society._members(members))));
}
