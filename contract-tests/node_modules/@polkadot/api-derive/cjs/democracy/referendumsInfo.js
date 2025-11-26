"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports._referendumVotes = _referendumVotes;
exports._referendumsVotes = _referendumsVotes;
exports._referendumInfo = _referendumInfo;
exports.referendumsInfo = referendumsInfo;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const util_js_1 = require("./util.js");
function votesPrev(api, referendumId) {
    return api.query.democracy['votersFor'](referendumId).pipe((0, rxjs_1.switchMap)((votersFor) => (0, rxjs_1.combineLatest)([
        (0, rxjs_1.of)(votersFor),
        votersFor.length
            ? api.query.democracy['voteOf'].multi(votersFor.map((accountId) => [referendumId, accountId]))
            : (0, rxjs_1.of)([]),
        api.derive.balances.votingBalances(votersFor)
    ])), (0, rxjs_1.map)(([votersFor, votes, balances]) => votersFor.map((accountId, index) => ({
        accountId,
        balance: balances[index].votingBalance || api.registry.createType('Balance'),
        isDelegating: false,
        vote: votes[index] || api.registry.createType('Vote')
    }))));
}
function extractVotes(mapped, referendumId) {
    return mapped
        .filter(([, voting]) => voting.isDirect)
        .map(([accountId, voting]) => [
        accountId,
        voting.asDirect.votes.filter(([idx]) => idx.eq(referendumId))
    ])
        .filter(([, directVotes]) => !!directVotes.length)
        .reduce((result, [accountId, votes]) => 
    // FIXME We are ignoring split votes
    votes.reduce((result, [, vote]) => {
        if (vote.isStandard) {
            result.push((0, util_1.objectSpread)({
                accountId,
                isDelegating: false
            }, vote.asStandard));
        }
        return result;
    }, result), []);
}
function votesCurr(api, referendumId) {
    return api.query.democracy.votingOf.entries().pipe((0, rxjs_1.map)((allVoting) => {
        const mapped = allVoting.map(([{ args: [accountId] }, voting]) => [accountId, voting]);
        const votes = extractVotes(mapped, referendumId);
        const delegations = mapped
            .filter(([, voting]) => voting.isDelegating)
            .map(([accountId, voting]) => [accountId, voting.asDelegating]);
        // add delegations
        delegations.forEach(([accountId, { balance, conviction, target }]) => {
            // Are we delegating to a delegator
            const toDelegator = delegations.find(([accountId]) => accountId.eq(target));
            const to = votes.find(({ accountId }) => accountId.eq(toDelegator ? toDelegator[0] : target));
            // this delegation has a target
            if (to) {
                votes.push({
                    accountId,
                    balance,
                    isDelegating: true,
                    vote: api.registry.createType('Vote', { aye: to.vote.isAye, conviction })
                });
            }
        });
        return votes;
    }));
}
function _referendumVotes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (referendum) => (0, rxjs_1.combineLatest)([
        api.derive.democracy.sqrtElectorate(),
        (0, util_1.isFunction)(api.query.democracy.votingOf)
            ? votesCurr(api, referendum.index)
            : votesPrev(api, referendum.index)
    ]).pipe((0, rxjs_1.map)(([sqrtElectorate, votes]) => (0, util_js_1.calcVotes)(sqrtElectorate, referendum, votes))));
}
function _referendumsVotes(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (referendums) => referendums.length
        ? (0, rxjs_1.combineLatest)(referendums.map((referendum) => api.derive.democracy._referendumVotes(referendum)))
        : (0, rxjs_1.of)([]));
}
function _referendumInfo(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (index, info) => {
        const status = (0, util_js_1.getStatus)(info);
        return status
            ? api.derive.democracy.preimage(status.proposal ||
                status.proposalHash).pipe((0, rxjs_1.map)((image) => ({
                image,
                imageHash: (0, util_js_1.getImageHash)(status),
                index: api.registry.createType('ReferendumIndex', index),
                status
            })))
            : (0, rxjs_1.of)(null);
    });
}
/**
 * @name referendumsInfo
 * @description Retrieves information about multiple referendums by their IDs.
 * @param {BN[]} ids An array of referendum IDs to query.
 * @example
 * ```javascript
 * import { BN } from "@polkadot/util";
 *
 * const referendumIds = [new BN(1)];
 * const referendums = await api.derive.democracy.referendumsInfo(referendumIds);
 * console.log("Referendums Info:", referendums);
 * ```
 */
function referendumsInfo(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (ids) => ids.length
        ? api.query.democracy.referendumInfoOf.multi(ids).pipe((0, rxjs_1.switchMap)((infos) => (0, rxjs_1.combineLatest)(ids.map((id, index) => api.derive.democracy._referendumInfo(id, infos[index])))), (0, rxjs_1.map)((infos) => infos.filter((r) => !!r)))
        : (0, rxjs_1.of)([]));
}
