"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.prime = exports.proposals = exports.proposalHashes = exports.proposalCount = exports.proposal = exports.hasProposals = exports.members = void 0;
const tslib_1 = require("tslib");
const index_js_1 = require("../collective/index.js");
tslib_1.__exportStar(require("./votes.js"), exports);
tslib_1.__exportStar(require("./votesOf.js"), exports);
/**
 * @name members
 * @description Retrieves the list of members in the "council" collective.
 * @example
 * ```javascript
 * const members = await api.derive.council.members();
 * console.log(`Members: ${JSON.stringify(members)});
 * ```
 */
exports.members = (0, index_js_1.members)('council');
/**
 * @name hasProposals
 * @description Checks if there are any active proposals in the "council" collective.
 * @example
 * ```javascript
 * const exists = await api.derive.council.hasProposals();
 * console.log(exists);
 * ```
 */
exports.hasProposals = (0, index_js_1.hasProposals)('council');
/**
 * @name proposal
 * @description Retrieves details of a specific proposal in the "councilMotion" collective by its hash.
 * @example
 * ```javascript
 * const proposalDetails = await api.derive.council.proposal(PROPOSAL_HASH);
 * console.log(proposalDetails);
 * ```
 */
exports.proposal = (0, index_js_1.proposal)('council');
/**
 * @name proposalCount
 * @description Retrieves the total number of proposals in the "council" collective.
 * @example
 * ```javascript
 * const count = await api.derive.council.proposalCount();
 * console.log(`Amount of proposals: ${count}`);
 * ```
 */
exports.proposalCount = (0, index_js_1.proposalCount)('council');
/**
 * @name proposalHashes
 * @description Retrieves an array of hashes for all active proposals in the "council" collective.
 * @example
 * ```javascript
 * const hashes = await api.derive.council.proposalHashes();
 * console.log(`Proposals ${JSON.stringify(hashes)}`);
 * ```
 */
exports.proposalHashes = (0, index_js_1.proposalHashes)('council');
/**
 * @name proposals
 * @description Retrieves a list of all active proposals in the "council" collective.
 * @example
 * ```javascript
 * const proposals = await api.derive.council.proposals();
 * console.log(proposals);
 * ```
 */
exports.proposals = (0, index_js_1.proposals)('council');
/**
 * @name prime
 * @description Retrieves the prime member of the "council" collective, if one exists.
 * @example
 * ```javascript
 * const primeMember = await api.derive.council.prime();
 * console.log(primeMember);
 * ```
 */
exports.prime = (0, index_js_1.prime)('council');
