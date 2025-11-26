import { combineLatest, map, of, switchMap } from 'rxjs';
import { memo } from '../util/index.js';
function parseResult(api, { allIds, allProposals, approvalIds, councilProposals, proposalCount }) {
    const approvals = [];
    const proposals = [];
    const councilTreasury = councilProposals.filter(({ proposal }) => 
    // FIXME `approveProposal` and `rejectProposal` have been removed in substrate and released in 1.14
    // in favor of `spend`. See: https://github.com/paritytech/polkadot-sdk/pull/3820
    proposal && ((api.tx.treasury['approveProposal'] && api.tx.treasury['approveProposal'].is(proposal)) ||
        (api.tx.treasury['rejectProposal'] && api.tx.treasury['rejectProposal'].is(proposal))));
    allIds.forEach((id, index) => {
        if (allProposals[index].isSome) {
            const council = councilTreasury
                .filter(({ proposal }) => proposal && id.eq(proposal.args[0]))
                .sort((a, b) => a.proposal && b.proposal
                ? a.proposal.method.localeCompare(b.proposal.method)
                : a.proposal
                    ? -1
                    : 1);
            const isApproval = approvalIds.some((approvalId) => approvalId.eq(id));
            const derived = { council, id, proposal: allProposals[index].unwrap() };
            if (isApproval) {
                approvals.push(derived);
            }
            else {
                proposals.push(derived);
            }
        }
    });
    return { approvals, proposalCount, proposals };
}
function retrieveProposals(api, proposalCount, approvalIds) {
    const proposalIds = [];
    const count = proposalCount.toNumber();
    for (let index = 0; index < count; index++) {
        if (!approvalIds.some((id) => id.eqn(index))) {
            proposalIds.push(api.registry.createType('ProposalIndex', index));
        }
    }
    const allIds = [...proposalIds, ...approvalIds];
    return combineLatest([
        api.query.treasury.proposals.multi(allIds),
        api.derive.council
            ? api.derive.council.proposals()
            : of([])
    ]).pipe(map(([allProposals, councilProposals]) => parseResult(api, { allIds, allProposals, approvalIds, councilProposals, proposalCount })));
}
/**
 * @name proposals
 * @description Retrieve all active and approved treasury proposals, along with their info.
 * @example
 * ```javascript
 * const treasuryProposals = await api.derive.treasury.proposals();
 * console.log(treasuryProposals);
 * ```
 */
export function proposals(instanceId, api) {
    return memo(instanceId, () => api.query.treasury
        ? combineLatest([
            api.query.treasury.proposalCount(),
            api.query.treasury.approvals()
        ]).pipe(switchMap(([proposalCount, approvalIds]) => retrieveProposals(api, proposalCount, approvalIds)))
        : of({
            approvals: [],
            proposalCount: api.registry.createType('ProposalIndex'),
            proposals: []
        }));
}
