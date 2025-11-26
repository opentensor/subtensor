"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.info = info;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
function isSeatHolder(value) {
    return !Array.isArray(value);
}
function isCandidateTuple(value) {
    return Array.isArray(value);
}
function getAccountTuple(value) {
    return isSeatHolder(value)
        ? [value.who, value.stake]
        : value;
}
function getCandidate(value) {
    return isCandidateTuple(value)
        ? value[0]
        : value;
}
function sortAccounts([, balanceA], [, balanceB]) {
    return balanceB.cmp(balanceA);
}
function getConstants(api, elections) {
    return elections
        ? {
            candidacyBond: api.consts[elections].candidacyBond,
            desiredRunnersUp: api.consts[elections].desiredRunnersUp,
            desiredSeats: api.consts[elections].desiredMembers,
            termDuration: api.consts[elections].termDuration,
            votingBond: api.consts[elections]['votingBond'],
            votingBondBase: api.consts[elections].votingBondBase,
            votingBondFactor: api.consts[elections].votingBondFactor
        }
        : {};
}
function getModules(api) {
    const [council] = api.registry.getModuleInstances(api.runtimeVersion.specName, 'council') || ['council'];
    const elections = api.query['phragmenElection']
        ? 'phragmenElection'
        : api.query['electionsPhragmen']
            ? 'electionsPhragmen'
            : api.query.elections
                ? 'elections'
                : null;
    // In some cases council here can refer to `generalCouncil` depending on what the chain specific override is.
    // Therefore, we check to see if it exists in the query field. If it does not we default to `council`.
    const resolvedCouncil = api.query[council] ? council : 'council';
    return [resolvedCouncil, elections];
}
function queryAll(api, council, elections) {
    return api.queryMulti([
        api.query[council].members,
        api.query[elections].candidates,
        api.query[elections].members,
        api.query[elections].runnersUp
    ]);
}
function queryCouncil(api, council) {
    return (0, rxjs_1.combineLatest)([
        api.query[council].members(),
        (0, rxjs_1.of)([]),
        (0, rxjs_1.of)([]),
        (0, rxjs_1.of)([])
    ]);
}
/**
 * @name info
 * @description An object containing the combined results of the storage queries for all relevant election module properties.
 * @example
 * ```javascript
 * api.derive.elections.info(({ members, candidates }) => {
 *   console.log(`There are currently ${members.length} council members and ${candidates.length} prospective council candidates.`);
 * });
 * ```
 */
function info(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => {
        const [council, elections] = getModules(api);
        return (elections
            ? queryAll(api, council, elections)
            : queryCouncil(api, council)).pipe((0, rxjs_1.map)(([councilMembers, candidates, members, runnersUp]) => (0, util_1.objectSpread)({}, getConstants(api, elections), {
            candidateCount: api.registry.createType('u32', candidates.length),
            candidates: candidates.map(getCandidate),
            members: members.length
                ? members.map(getAccountTuple).sort(sortAccounts)
                : councilMembers.map((a) => [a, api.registry.createType('Balance')]),
            runnersUp: runnersUp.map(getAccountTuple).sort(sortAccounts)
        })));
    });
}
