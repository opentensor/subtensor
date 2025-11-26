"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.votes = votes;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
function isVoter(value) {
    return !Array.isArray(value);
}
function retrieveStakeOf(elections) {
    return elections['stakeOf'].entries().pipe((0, rxjs_1.map)((entries) => entries.map(([{ args: [accountId] }, stake]) => [accountId, stake])));
}
function retrieveVoteOf(elections) {
    return elections['votesOf'].entries().pipe((0, rxjs_1.map)((entries) => entries.map(([{ args: [accountId] }, votes]) => [accountId, votes])));
}
function retrievePrev(api, elections) {
    return (0, rxjs_1.combineLatest)([
        retrieveStakeOf(elections),
        retrieveVoteOf(elections)
    ]).pipe((0, rxjs_1.map)(([stakes, votes]) => {
        const result = [];
        votes.forEach(([voter, votes]) => {
            result.push([voter, { stake: api.registry.createType('Balance'), votes }]);
        });
        stakes.forEach(([staker, stake]) => {
            const entry = result.find(([voter]) => voter.eq(staker));
            if (entry) {
                entry[1].stake = stake;
            }
            else {
                result.push([staker, { stake, votes: [] }]);
            }
        });
        return result;
    }));
}
function retrieveCurrent(elections) {
    return elections.voting.entries().pipe((0, rxjs_1.map)((entries) => entries.map(([{ args: [accountId] }, value]) => [
        accountId,
        isVoter(value)
            ? { stake: value.stake, votes: value.votes }
            : { stake: value[0], votes: value[1] }
    ])));
}
/**
 * @name votes
 * @description Retrieves the council election votes for all participants.
 * @example
 * ```javascript
 * const votes = await api.derive.council.votes();
 * ```
 */
function votes(instanceId, api) {
    const elections = api.query.elections || api.query['phragmenElection'] || api.query['electionsPhragmen'];
    return (0, index_js_1.memo)(instanceId, () => elections
        ? elections['stakeOf']
            ? retrievePrev(api, elections)
            : retrieveCurrent(elections)
        : (0, rxjs_1.of)([]));
}
