import { map, of } from 'rxjs';
import { isFunction } from '@polkadot/util';
import { memo } from '../util/index.js';
function parseFlags(address, [electionsMembers, councilMembers, technicalCommitteeMembers, societyMembers, sudoKey]) {
    const addrStr = address?.toString();
    const isIncluded = (id) => id.toString() === addrStr;
    return {
        isCouncil: (electionsMembers?.map((r) => Array.isArray(r) ? r[0] : r.who) || councilMembers || []).some(isIncluded),
        isSociety: (societyMembers || []).some(isIncluded),
        isSudo: sudoKey?.toString() === addrStr,
        isTechCommittee: (technicalCommitteeMembers || []).some(isIncluded)
    };
}
export function _flags(instanceId, api) {
    return memo(instanceId, () => {
        const results = [undefined, [], [], [], undefined];
        const calls = [
            (api.query.elections || api.query['phragmenElection'] || api.query['electionsPhragmen'])?.members,
            api.query.council?.members,
            api.query.technicalCommittee?.members,
            api.query.society?.members,
            api.query.sudo?.key
        ];
        const filtered = calls.filter((c) => c);
        if (!filtered.length) {
            return of(results);
        }
        return api.queryMulti(filtered).pipe(map((values) => {
            let resultIndex = -1;
            for (let i = 0, count = calls.length; i < count; i++) {
                if (isFunction(calls[i])) {
                    results[i] = values[++resultIndex];
                }
            }
            return results;
        }));
    });
}
/**
 * @name flags
 * @param {(AccountId | Address | string | null)} address The account identifier.
 * @description Retrieves the membership flags for a given account.
 * @example
 * const ALICE = "F7Hs";
 *
 * api.derive.accounts.flags(ALICE, (flags) => {
 *   console.log(
 *     `Account Flags:`,
 *     Object.keys(flags).map((flag) => `${flag}: ${flags[flag]}`)
 *   );
 * });
 */
export function flags(instanceId, api) {
    return memo(instanceId, (address) => api.derive.accounts._flags().pipe(map((r) => parseFlags(address, r))));
}
