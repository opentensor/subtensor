"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.locks = locks;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const LOCKUPS = [0, 1, 2, 4, 8, 16, 32];
function parseEnd(api, vote, { approved, end }) {
    return [
        end,
        (approved.isTrue && vote.isAye) || (approved.isFalse && vote.isNay)
            ? end.add((api.consts.democracy.voteLockingPeriod ||
                api.consts.democracy.enactmentPeriod).muln(LOCKUPS[vote.conviction.index]))
            : util_1.BN_ZERO
    ];
}
function parseLock(api, [referendumId, accountVote], referendum) {
    const { balance, vote } = accountVote.asStandard;
    const [referendumEnd, unlockAt] = referendum.isFinished
        ? parseEnd(api, vote, referendum.asFinished)
        : [util_1.BN_ZERO, util_1.BN_ZERO];
    return { balance, isDelegated: false, isFinished: referendum.isFinished, referendumEnd, referendumId, unlockAt, vote };
}
function delegateLocks(api, { balance, conviction, target }) {
    return api.derive.democracy.locks(target).pipe((0, rxjs_1.map)((available) => available.map(({ isFinished, referendumEnd, referendumId, unlockAt, vote }) => ({
        balance,
        isDelegated: true,
        isFinished,
        referendumEnd,
        referendumId,
        unlockAt: unlockAt.isZero()
            ? unlockAt
            : referendumEnd.add((api.consts.democracy.voteLockingPeriod ||
                api.consts.democracy.enactmentPeriod).muln(LOCKUPS[conviction.index])),
        vote: api.registry.createType('Vote', { aye: vote.isAye, conviction })
    }))));
}
function directLocks(api, { votes }) {
    if (!votes.length) {
        return (0, rxjs_1.of)([]);
    }
    return api.query.democracy.referendumInfoOf.multi(votes.map(([referendumId]) => referendumId)).pipe((0, rxjs_1.map)((referendums) => votes
        .map((vote, index) => [vote, referendums[index].unwrapOr(null)])
        .filter((item) => !!item[1] && (0, util_1.isUndefined)(item[1].end) && item[0][1].isStandard)
        .map(([directVote, referendum]) => parseLock(api, directVote, referendum))));
}
/**
 * @name locks
 * @description Retrieves the democracy voting locks for a given account.
 * @param { string | AccountId } accountId The accountId for which to retrieve democracy voting locks.
 * @example
 * ```javascript
 * const locks = await api.derive.democracy.locks('5FfFjX...'); // Replace with an actual accountId
 * console.log("Democracy Locks:", locks);
 * ```
 */
function locks(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId) => api.query.democracy.votingOf
        ? api.query.democracy.votingOf(accountId).pipe((0, rxjs_1.switchMap)((voting) => voting.isDirect
            ? directLocks(api, voting.asDirect)
            : voting.isDelegating
                ? delegateLocks(api, voting.asDelegating)
                : (0, rxjs_1.of)([])))
        : (0, rxjs_1.of)([]));
}
