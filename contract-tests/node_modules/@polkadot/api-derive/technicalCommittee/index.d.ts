/**
 * @name members
 * @description Retrieves the list of members in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const members = await api.derive.technicalCommittee.members();
 * console.log(`Members: ${JSON.stringify(members)});
 * ```
 */
export declare const members: import("../collective/types.js").MembersFnRet;
/**
 * @name hasProposals
 * @description Checks if there are any active proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const exists = await api.derive.technicalCommittee.hasProposals();
 * console.log(exists);
 * ```
 */
export declare const hasProposals: import("../collective/types.js").HasProposalsFnRet;
/**
 * @name proposal
 * @description Retrieves details of a specific proposal in the "technicalCommitteeMotion" collective by its hash.
 * @example
 * ```javascript
 * const proposalDetails = await api.derive.technicalCommittee.proposal(PROPOSAL_HASH);
 * console.log(proposalDetails);
 * ```
 */
export declare const proposal: import("../collective/types.js").ProposalFnRet;
/**
 * @name proposalCount
 * @description Retrieves the total number of proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const count = await api.derive.technicalCommittee.proposalCount();
 * console.log(`Amount of proposals: ${count}`);
 * ```
 */
export declare const proposalCount: import("../collective/types.js").ProposalCountFnRet;
/**
 * @name proposalHashes
 * @description Retrieves an array of hashes for all active proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const hashes = await api.derive.technicalCommittee.proposalHashes();
 * console.log(`Proposals ${JSON.stringify(hashes)}`);
 * ```
 */
export declare const proposalHashes: import("../collective/types.js").ProposalHashesFnRet;
/**
 * @name proposals
 * @description Retrieves a list of all active proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const proposals = await api.derive.technicalCommittee.proposals();
 * console.log(proposals);
 * ```
 */
export declare const proposals: import("../collective/types.js").ProposalsFnRet;
/**
 * @name prime
 * @description Retrieves the prime member of the "technicalCommittee" collective, if one exists.
 * @example
 * ```javascript
 * const primeMember = await api.derive.technicalCommittee.prime();
 * console.log(primeMember);
 * ```
 */
export declare const prime: import("../collective/types.js").PrimeFnRet;
