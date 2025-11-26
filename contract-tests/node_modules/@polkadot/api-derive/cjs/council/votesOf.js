"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.votesOf = votesOf;
const rxjs_1 = require("rxjs");
const index_js_1 = require("../util/index.js");
/**
 * @name votesOf
 * @description Retrieves the council votes associated with a given account.
 * @returns The stake and the list of candidates the account has voted for.
 * @param {string | Uint8Array | AccountId} accountId The accountId to retrieve votes for.
 * @example
 * ```javascript
 * const accountId = "5Gw3s7qQ9Z..."; // Replace with a valid account ID
 * const votes = await api.derive.council.votesOf(accountId);
 * console.log("Account votes:", votes);
 * ```
 */
function votesOf(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (accountId) => api.derive.council.votes().pipe((0, rxjs_1.map)((votes) => (votes.find(([from]) => from.eq(accountId)) ||
        [null, { stake: api.registry.createType('Balance'), votes: [] }])[1])));
}
