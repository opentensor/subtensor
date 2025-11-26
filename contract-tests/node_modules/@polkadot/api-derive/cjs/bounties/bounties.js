"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.bounties = bounties;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
const filterBountyProposals_js_1 = require("./helpers/filterBountyProposals.js");
function parseResult([maybeBounties, maybeDescriptions, ids, bountyProposals]) {
    const bounties = [];
    maybeBounties.forEach((bounty, index) => {
        if (bounty.isSome) {
            bounties.push({
                bounty: bounty.unwrap(),
                description: maybeDescriptions[index].unwrapOrDefault().toUtf8(),
                index: ids[index],
                proposals: bountyProposals.filter((bountyProposal) => bountyProposal.proposal && ids[index].eq(bountyProposal.proposal.args[0]))
            });
        }
    });
    return bounties;
}
/**
 * @name bounties
 * @descrive Retrieves all active bounties, their descriptions, and associated proposals.
 * @example
 * ```javascript
 * const bounties = await api.derive.bounties();
 * console.log("Active bounties:", bounties);
 * ```
 */
function bounties(instanceId, api) {
    const bountyBase = api.query.bounties || api.query.treasury;
    return (0, index_js_1.memo)(instanceId, () => bountyBase.bounties
        ? (0, rxjs_1.combineLatest)([
            bountyBase.bountyCount(),
            api.query.council
                ? api.query.council.proposalCount()
                : (0, rxjs_1.of)(0)
        ]).pipe((0, rxjs_1.switchMap)(() => (0, rxjs_1.combineLatest)([
            bountyBase.bounties.keys(),
            api.derive.council
                ? api.derive.council.proposals()
                : (0, rxjs_1.of)([])
        ])), (0, rxjs_1.switchMap)(([keys, proposals]) => {
            const ids = keys.map(({ args: [id] }) => id);
            return (0, rxjs_1.combineLatest)([
                bountyBase.bounties.multi(ids),
                bountyBase.bountyDescriptions.multi(ids),
                (0, rxjs_1.of)(ids),
                (0, rxjs_1.of)((0, filterBountyProposals_js_1.filterBountiesProposals)(api, proposals))
            ]);
        }), (0, rxjs_1.map)(parseResult))
        : (0, rxjs_1.of)(parseResult([[], [], [], []])));
}
