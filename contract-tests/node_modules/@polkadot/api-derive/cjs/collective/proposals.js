"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.proposalHashes = exports.proposalCount = void 0;
exports.hasProposals = hasProposals;
exports.proposals = proposals;
exports.proposal = proposal;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
const helpers_js_1 = require("./helpers.js");
function parse(api, [hashes, proposals, votes]) {
    return proposals.map((o, index) => ({
        hash: api.registry.createType('Hash', hashes[index]),
        proposal: o && o.isSome
            ? o.unwrap()
            : null,
        votes: votes[index].unwrapOr(null)
    }));
}
function _proposalsFrom(api, query, hashes) {
    return ((0, util_1.isFunction)(query?.proposals) && hashes.length
        ? (0, rxjs_1.combineLatest)([
            (0, rxjs_1.of)(hashes),
            // this should simply be api.query[section].proposalOf.multi<Option<Proposal>>(hashes),
            // however we have had cases on Edgeware where the indices have moved around after an
            // upgrade, which results in invalid on-chain data
            query.proposalOf.multi(hashes).pipe((0, rxjs_1.catchError)(() => (0, rxjs_1.of)(hashes.map(() => null)))),
            query.voting.multi(hashes)
        ])
        : (0, rxjs_1.of)([[], [], []])).pipe((0, rxjs_1.map)((r) => parse(api, r)));
}
function hasProposals(section) {
    return (0, helpers_js_1.withSection)(section, (query) => () => (0, rxjs_1.of)((0, util_1.isFunction)(query?.proposals)));
}
function proposals(section) {
    return (0, helpers_js_1.withSection)(section, (query, api) => () => api.derive[section].proposalHashes().pipe((0, rxjs_1.switchMap)((all) => _proposalsFrom(api, query, all))));
}
function proposal(section) {
    return (0, helpers_js_1.withSection)(section, (query, api) => (hash) => (0, util_1.isFunction)(query?.proposals)
        ? (0, index_js_1.firstObservable)(_proposalsFrom(api, query, [hash]))
        : (0, rxjs_1.of)(null));
}
exports.proposalCount = (0, helpers_js_1.callMethod)('proposalCount', null);
exports.proposalHashes = (0, helpers_js_1.callMethod)('proposals', []);
