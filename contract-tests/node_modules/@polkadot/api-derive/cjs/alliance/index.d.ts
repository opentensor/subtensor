/**
 * @name members
 * @description Retrieves the list of members in the "allianceMotion" collective.
 * @example
 * ```javascript
 * const members = await api.derive.alliance.members();
 * console.log(`Members: ${JSON.stringify(members)});
 * ```
 */
export declare const members: import("../collective/types.js").MembersFnRet;
/**
 * @name hasProposals
 * @description Checks if there are any active proposals in the "allianceMotion" collective.
 * @example
 * ```javascript
 * const exists = await api.derive.alliance.hasProposals();
 * console.log(exists);
 * ```
 */
export declare const hasProposals: import("../collective/types.js").HasProposalsFnRet;
/**
 * @name proposal
 * @description Retrieves details of a specific proposal in the "allianceMotion" collective by its hash.
 * @example
 * ```javascript
 * const proposalDetails = await api.derive.alliance.proposal(PROPOSAL_HASH);
 * console.log(proposalDetails);
 * ```
 */
export declare const proposal: import("../collective/types.js").ProposalFnRet;
/**
 * @name proposalCount
 * @description Retrieves the total number of proposals in the "allianceMotion" collective.
 * @example
 * ```javascript
 * const count = await api.derive.alliance.proposalCount();
 * console.log(`Amount of proposals: ${count}`);
 * ```
 */
export declare const proposalCount: import("../collective/types.js").ProposalCountFnRet;
/**
 * @name proposalHashes
 * @description Retrieves an array of hashes for all active proposals in the "allianceMotion" collective.
 * @example
 * ```javascript
 * const hashes = await api.derive.alliance.proposalHashes();
 * console.log(`Proposals ${JSON.stringify(hashes)}`);
 * ```
 */
export declare const proposalHashes: import("../collective/types.js").ProposalHashesFnRet;
/**
 * @name proposals
 * @description Retrieves a list of all active proposals in the "allianceMotion" collective.
 * @example
 * ```javascript
 * const proposals = await api.derive.alliance.proposals();
 * console.log(proposals);
 * ```
 */
export declare const proposals: import("../collective/types.js").ProposalsFnRet;
/**
 * @name prime
 * @description Retrieves the prime member of the "allianceMotion" collective, if one exists.
 * @example
 * ```javascript
 * const primeMember = await api.derive.alliance.prime();
 * console.log(primeMember);
 * ```
 */
export declare const prime: import("../collective/types.js").PrimeFnRet;
