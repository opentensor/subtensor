"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.prime = exports.proposals = exports.proposalHashes = exports.proposalCount = exports.proposal = exports.hasProposals = exports.members = void 0;
const index_js_1 = require("../collective/index.js");
/**
 * @name members
 * @description Retrieves the list of members in the "membership" collective.
 * @example
 * ```javascript
 * const members = await api.derive.membership.members();
 * console.log(`Members: ${JSON.stringify(members)});
 * ```
 */
exports.members = (0, index_js_1.members)('membership');
/**
 * @name hasProposals
 * @description Checks if there are any active proposals in the "membership" collective.
 * @example
 * ```javascript
 * const exists = await api.derive.membership.hasProposals();
 * console.log(exists);
 * ```
 */
exports.hasProposals = (0, index_js_1.hasProposals)('membership');
/**
 * @name proposal
 * @description Retrieves details of a specific proposal in the "membership" collective by its hash.
 * @example
 * ```javascript
 * const proposalDetails = await api.derive.membership.proposal(PROPOSAL_HASH);
 * console.log(proposalDetails);
 * ```
 */
exports.proposal = (0, index_js_1.proposal)('membership');
/**
 * @name proposalCount
 * @description Retrieves the total number of proposals in the "membership" collective.
 * @example
 * ```javascript
 * const count = await api.derive.membership.proposalCount();
 * console.log(`Amount of proposals: ${count}`);
 * ```
 */
exports.proposalCount = (0, index_js_1.proposalCount)('membership');
/**
 * @name proposalHashes
 * @description Retrieves an array of hashes for all active proposals in the "membership" collective.
 * @example
 * ```javascript
 * const hashes = await api.derive.membership.proposalHashes();
 * console.log(`Proposals ${JSON.stringify(hashes)}`);
 * ```
 */
exports.proposalHashes = (0, index_js_1.proposalHashes)('membership');
/**
 * @name proposals
 * @description Retrieves a list of all active proposals in the "membership" collective.
 * @example
 * ```javascript
 * const proposals = await api.derive.membership.proposals();
 * console.log(proposals);
 * ```
 */
exports.proposals = (0, index_js_1.proposals)('membership');
/**
 * @name prime
 * @description Retrieves the prime member of the "membership" collective, if one exists.
 * @example
 * ```javascript
 * const primeMember = await api.derive.membership.prime();
 * console.log(primeMember);
 * ```
 */
exports.prime = (0, index_js_1.prime)('membership');
