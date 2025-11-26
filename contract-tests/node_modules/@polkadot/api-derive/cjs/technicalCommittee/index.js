"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.prime = exports.proposals = exports.proposalHashes = exports.proposalCount = exports.proposal = exports.hasProposals = exports.members = void 0;
const index_js_1 = require("../collective/index.js");
/**
 * @name members
 * @description Retrieves the list of members in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const members = await api.derive.technicalCommittee.members();
 * console.log(`Members: ${JSON.stringify(members)});
 * ```
 */
exports.members = (0, index_js_1.members)('technicalCommittee');
/**
 * @name hasProposals
 * @description Checks if there are any active proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const exists = await api.derive.technicalCommittee.hasProposals();
 * console.log(exists);
 * ```
 */
exports.hasProposals = (0, index_js_1.hasProposals)('technicalCommittee');
/**
 * @name proposal
 * @description Retrieves details of a specific proposal in the "technicalCommitteeMotion" collective by its hash.
 * @example
 * ```javascript
 * const proposalDetails = await api.derive.technicalCommittee.proposal(PROPOSAL_HASH);
 * console.log(proposalDetails);
 * ```
 */
exports.proposal = (0, index_js_1.proposal)('technicalCommittee');
/**
 * @name proposalCount
 * @description Retrieves the total number of proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const count = await api.derive.technicalCommittee.proposalCount();
 * console.log(`Amount of proposals: ${count}`);
 * ```
 */
exports.proposalCount = (0, index_js_1.proposalCount)('technicalCommittee');
/**
 * @name proposalHashes
 * @description Retrieves an array of hashes for all active proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const hashes = await api.derive.technicalCommittee.proposalHashes();
 * console.log(`Proposals ${JSON.stringify(hashes)}`);
 * ```
 */
exports.proposalHashes = (0, index_js_1.proposalHashes)('technicalCommittee');
/**
 * @name proposals
 * @description Retrieves a list of all active proposals in the "technicalCommittee" collective.
 * @example
 * ```javascript
 * const proposals = await api.derive.technicalCommittee.proposals();
 * console.log(proposals);
 * ```
 */
exports.proposals = (0, index_js_1.proposals)('technicalCommittee');
/**
 * @name prime
 * @description Retrieves the prime member of the "technicalCommittee" collective, if one exists.
 * @example
 * ```javascript
 * const primeMember = await api.derive.technicalCommittee.prime();
 * console.log(primeMember);
 * ```
 */
exports.prime = (0, index_js_1.prime)('technicalCommittee');
