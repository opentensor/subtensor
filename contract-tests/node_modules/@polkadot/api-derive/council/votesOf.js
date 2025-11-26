import { map } from 'rxjs';
import { memo } from '../util/index.js';
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
export function votesOf(instanceId, api) {
    return memo(instanceId, (accountId) => api.derive.council.votes().pipe(map((votes) => (votes.find(([from]) => from.eq(accountId)) ||
        [null, { stake: api.registry.createType('Balance'), votes: [] }])[1])));
}
