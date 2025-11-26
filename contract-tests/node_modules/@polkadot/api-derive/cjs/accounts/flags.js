"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports._flags = _flags;
exports.flags = flags;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const index_js_1 = require("../util/index.js");
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
function _flags(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, () => {
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
            return (0, rxjs_1.of)(results);
        }
        return api.queryMulti(filtered).pipe((0, rxjs_1.map)((values) => {
            let resultIndex = -1;
            for (let i = 0, count = calls.length; i < count; i++) {
                if ((0, util_1.isFunction)(calls[i])) {
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
function flags(instanceId, api) {
    return (0, index_js_1.memo)(instanceId, (address) => api.derive.accounts._flags().pipe((0, rxjs_1.map)((r) => parseFlags(address, r))));
}
